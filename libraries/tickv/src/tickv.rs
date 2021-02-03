//! The TicKV implementation.

use crate::crc32;
use crate::error_codes::ErrorCode;
use crate::flash_controller::FlashController;
use crate::success_codes::SuccessCode;
use core::cell::Cell;

/// The current version of TicKV
pub const VERSION: u8 = 0;

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum InitState {
    /// Trying to read the key from a region
    GetKeyReadRegion(usize),
    /// Trying to erase a region
    EraseRegion(usize),
    /// Finished erasing regions
    EraseComplete,
    /// Trying to read a region while appending a key
    AppendKeyReadRegion(usize),
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum KeyState {
    /// Trying to read the key from a region
    ReadRegion(usize),
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum RubbishState {
    ReadRegion(usize),
    EraseRegion(usize),
}

#[derive(Clone, Copy, PartialEq)]
/// The current state machine when trying to complete a previous operation.
/// This is used when returning from a complete async `FlashController` call.
pub(crate) enum State {
    /// No previous state
    None,
    /// Init Operation
    Init(InitState),
    /// Appending a key
    AppendKey(KeyState),
    /// Getting a key
    GetKey(KeyState),
    /// Invalidating a key
    InvalidateKey(KeyState),
    /// Running garbage collection
    GarbageCollect(RubbishState),
}

/// The struct storing all of the TicKV information.
pub struct TicKV<'a, C: FlashController<S>, const S: usize> {
    /// The controller used for flash commands
    pub controller: C,
    flash_size: usize,
    pub(crate) read_buffer: Cell<Option<&'a mut [u8; S]>>,
    pub(crate) state: Cell<State>,
}

/// This is the current object header used for TicKV objects
struct ObjectHeader {
    version: u8,
    // In reality this is a u4.
    flags: u8,
    // In reality this is a u12.
    len: u16,
    hashed_key: u64,
}

pub(crate) const FLAGS_VALID: u8 = 8;

impl ObjectHeader {
    fn new(hashed_key: u64, len: u16) -> Self {
        assert!(len < 0xFFF);
        Self {
            version: VERSION,
            flags: FLAGS_VALID,
            len,
            hashed_key,
        }
    }
}

// A list of offsets into the ObjectHeader
pub(crate) const VERSION_OFFSET: usize = 0;
pub(crate) const LEN_OFFSET: usize = 1;
pub(crate) const HASH_OFFSET: usize = 3;
pub(crate) const HEADER_LENGTH: usize = HASH_OFFSET + 8;
pub(crate) const CHECK_SUM_LEN: usize = 4;

/// The main key. A hashed version of this should be passed to
/// `initalise()`.
pub const MAIN_KEY: &[u8; 15] = b"tickv-super-key";

/// This is the main TicKV struct.
impl<'a, C: FlashController<S>, const S: usize> TicKV<'a, C, S> {
    /// Create a new struct
    ///
    /// `C`: An implementation of the `FlashController` trait
    ///
    /// `controller`: An new struct implementing `FlashController`
    /// `flash_size`: The total size of the flash used for TicKV
    pub fn new(controller: C, read_buffer: &'a mut [u8; S], flash_size: usize) -> Self {
        Self {
            controller,
            flash_size,
            read_buffer: Cell::new(Some(read_buffer)),
            state: Cell::new(State::None),
        }
    }

    /// This function setups the flash region to be used as a key-value store.
    /// If the region is already initalised this won't make any changes.
    ///
    /// `hashed_main_key`: The u64 hash of the const string `MAIN_KEY`.
    ///
    /// If the specified region has not already been setup for TicKV
    /// the entire region will be erased.
    ///
    /// On success nothing will be returned.
    /// On error a `ErrorCode` will be returned.
    pub fn initalise(&self, hashed_main_key: u64) -> Result<SuccessCode, ErrorCode> {
        let mut buf: [u8; 0] = [0; 0];

        let key_ret = match self.state.get() {
            State::None => self.get_key(hashed_main_key, &mut buf),
            State::Init(state) => match state {
                InitState::GetKeyReadRegion(_) => self.get_key(hashed_main_key, &mut buf),
                _ => Err(ErrorCode::EraseNotReady(0)),
            },
            _ => unreachable!(),
        };

        match key_ret {
            Ok(ret) => Ok(ret),
            Err(e) => {
                match e {
                    ErrorCode::ReadNotReady(reg) => {
                        self.state
                            .set(State::Init(InitState::GetKeyReadRegion(reg)));
                        Err(ErrorCode::ReadNotReady(reg))
                    }
                    _ => {
                        match self.state.get() {
                            State::None
                            | State::Init(InitState::GetKeyReadRegion(_))
                            | State::Init(InitState::EraseRegion(_)) => {
                                // Erase all regions
                                let mut start = 0;
                                if let State::Init(InitState::EraseRegion(reg)) = self.state.get() {
                                    // We already erased region reg, so move to the next one
                                    start = reg + 1;
                                }

                                if start < (self.flash_size / S) {
                                    for r in start..(self.flash_size / S) {
                                        match self.controller.erase_region(r) {
                                            Ok(()) => {}
                                            Err(e) => {
                                                self.state
                                                    .set(State::Init(InitState::EraseRegion(r)));
                                                return Err(e);
                                            }
                                        }
                                    }
                                }

                                self.state.set(State::Init(InitState::EraseComplete));
                            }
                            _ => {}
                        }

                        // Save the main key
                        match self.append_key(hashed_main_key, &buf) {
                            Ok(ret) => {
                                self.state.set(State::None);
                                Ok(ret)
                            }
                            Err(e) => match e {
                                ErrorCode::ReadNotReady(reg) => {
                                    self.state
                                        .set(State::Init(InitState::AppendKeyReadRegion(reg)));
                                    Err(e)
                                }
                                ErrorCode::WriteNotReady(_) => {
                                    self.state.set(State::None);
                                    Ok(SuccessCode::Queued)
                                }
                                _ => Err(e),
                            },
                        }
                    }
                }
            }
        }
    }

    /// Get region number from a hashed key
    fn get_region(&self, hash: u64) -> usize {
        assert_ne!(hash, 0xFFFF_FFFF_FFFF_FFFF);
        assert_ne!(hash, 0);

        // Determine the number of regions
        let num_region = self.flash_size / S;

        // Determine the block where the data should be
        (hash as usize & 0xFFFF) % num_region
    }

    // Determine the new region offset to try.
    // Returns None if there aren't any more in range.
    fn increment_region_offset(&self, region_offset: isize) -> Option<isize> {
        let mut too_big = false;
        let mut too_small = false;
        // Loop until we find a region we can use
        while !too_big && !too_small {
            let new_offset = match region_offset {
                0 => 1,
                region_offset if region_offset > 0 => -region_offset,
                region_offset if region_offset < 0 => -region_offset + 1,
                _ => unreachable!(),
            };

            // Make sure our new offset is valid
            if new_offset as usize > ((self.flash_size / S) - 1) {
                too_big = true;
                continue;
            }

            if new_offset < 0 {
                too_small = true;
                continue;
            }

            return Some(new_offset);
        }

        None
    }

    /// Find a key in some loaded region data.
    ///
    /// On success return the offset in the region_data where the key is and the
    /// total length of the key.
    /// On failure return a bool indicating if the caller should keep looking in
    /// neighboring regions and the error code.
    fn find_key_offset(
        &self,
        hash: u64,
        region_data: &[u8],
    ) -> Result<(usize, u16), (bool, ErrorCode)> {
        // Determine the total size of our payload

        // Split the hash
        let hash = hash.to_ne_bytes();

        let mut offset: usize = 0;
        let mut empty: bool = true;

        loop {
            if offset + HEADER_LENGTH >= S {
                // We have reached the end of the region
                return Err((false, ErrorCode::KeyNotFound));
            }

            // Check to see if we have data
            if region_data[offset + VERSION_OFFSET] != 0xFF {
                // Mark that this region isn't empty
                empty = false;

                // We found a version, check that we support it
                if region_data[offset + VERSION_OFFSET] != VERSION {
                    return Err((false, ErrorCode::UnsupportedVersion));
                }

                // Find this entries length
                let total_length = ((region_data[offset + LEN_OFFSET] as u16) & !0xF0) << 8
                    | region_data[offset + LEN_OFFSET + 1] as u16;

                // Check to see if all fields are just 0
                if total_length == 0 {
                    // We found something invalid here
                    return Err((false, ErrorCode::KeyNotFound));
                }

                // Check to see if the entry has been deleted
                if region_data[offset + LEN_OFFSET] & 0x80 != 0x80 {
                    // Increment our offset by the length and repeat the loop
                    offset += total_length as usize;
                    continue;
                }

                // We have found a valid entry, see if it is ours.
                if region_data[offset + HASH_OFFSET] != hash[7]
                    || region_data[offset + HASH_OFFSET + 1] != hash[6]
                    || region_data[offset + HASH_OFFSET + 2] != hash[5]
                    || region_data[offset + HASH_OFFSET + 3] != hash[4]
                    || region_data[offset + HASH_OFFSET + 4] != hash[3]
                    || region_data[offset + HASH_OFFSET + 5] != hash[2]
                    || region_data[offset + HASH_OFFSET + 6] != hash[1]
                    || region_data[offset + HASH_OFFSET + 7] != hash[0]
                {
                    // Increment our offset by the length and repeat the loop
                    offset += total_length as usize;
                    continue;
                }

                // If we get here we have found out value (assuming no collisions)
                return Ok((offset, total_length));
            } else {
                // We hit the end.
                return Err((!empty, ErrorCode::KeyNotFound));
            }
        }
    }

    /// Appends the key/value pair to flash storage.
    ///
    /// `hash`: A hashed key. This key will be used in future to retrieve
    ///         or remove the `value`.
    /// `value`: A buffer containing the data to be stored to flash.
    ///
    /// On success nothing will be returned.
    /// On error a `ErrorCode` will be returned.
    pub fn append_key(&self, hash: u64, value: &[u8]) -> Result<SuccessCode, ErrorCode> {
        let region = self.get_region(hash);
        let crc = crc32::Crc::new();
        let mut check_sum = crc.digest();

        // Length not including check sum
        let package_length = HEADER_LENGTH + value.len();
        let object_length = HEADER_LENGTH + value.len() + CHECK_SUM_LEN;

        if object_length > 0xFFF {
            return Err(ErrorCode::ObjectTooLarge);
        }

        // Create the header:
        let header = ObjectHeader::new(hash, object_length as u16);

        let mut region_offset: isize = 0;

        loop {
            let new_region = match self.state.get() {
                State::None => region as isize + region_offset,
                State::Init(state) => {
                    match state {
                        InitState::AppendKeyReadRegion(reg) => reg as isize,
                        _ => {
                            // Get the data from that region
                            region as isize + region_offset
                        }
                    }
                }
                State::AppendKey(key_state) => match key_state {
                    KeyState::ReadRegion(reg) => reg as isize,
                },
                State::GarbageCollect(RubbishState::ReadRegion(reg)) => reg as isize,
                _ => unreachable!(),
            };

            let mut region_data = self.read_buffer.take().unwrap();
            if self.state.get() != State::AppendKey(KeyState::ReadRegion(new_region as usize))
                && self.state.get()
                    != State::Init(InitState::AppendKeyReadRegion(new_region as usize))
            {
                match self
                    .controller
                    .read_region(new_region as usize, 0, &mut region_data)
                {
                    Ok(()) => {}
                    Err(e) => {
                        self.read_buffer.replace(Some(region_data));
                        if let ErrorCode::ReadNotReady(reg) = e {
                            self.state.set(State::AppendKey(KeyState::ReadRegion(reg)));
                        }
                        return Err(e);
                    }
                };
            }

            if self.find_key_offset(hash, region_data).is_ok() {
                // Check to make sure we don't already have this key
                self.read_buffer.replace(Some(region_data));
                return Err(ErrorCode::KeyAlreadyExists);
            }

            let mut offset: usize = 0;

            loop {
                if offset + package_length >= S {
                    // We have reached the end of the region
                    // We will need to try the next region

                    // Replace the buffer
                    self.read_buffer.replace(Some(region_data));

                    match self.increment_region_offset(new_region) {
                        Some(o) => {
                            region_offset = o;
                        }
                        None => {
                            return Err(ErrorCode::FlashFull);
                        }
                    }
                    break;
                }

                // Check to see if we have data
                if region_data[offset + VERSION_OFFSET] != 0xFF {
                    // We found a version, check that we support it
                    if region_data[offset + VERSION_OFFSET] != VERSION {
                        self.read_buffer.replace(Some(region_data));
                        return Err(ErrorCode::UnsupportedVersion);
                    }

                    // Find this entries length
                    let total_length = ((region_data[offset + LEN_OFFSET] as u16) & !0xF0) << 8
                        | region_data[offset + LEN_OFFSET + 1] as u16;

                    // Increment our offset by the length and repeat the loop
                    offset += total_length as usize;
                    continue;
                }

                // If we get here we have found an empty spot
                // Double check that there is no valid hash

                // Check to see if the entire header is 0xFFFF_FFFF_FFFF_FFFF
                // To avoid operating on 64-bit values check every 8 bytes at a time
                if region_data[offset + HASH_OFFSET] != 0xFF {
                    self.read_buffer.replace(Some(region_data));
                    return Err(ErrorCode::CorruptData);
                }
                if region_data[offset + HASH_OFFSET + 1] != 0xFF {
                    self.read_buffer.replace(Some(region_data));
                    return Err(ErrorCode::CorruptData);
                }
                if region_data[offset + HASH_OFFSET + 2] != 0xFF {
                    self.read_buffer.replace(Some(region_data));
                    return Err(ErrorCode::CorruptData);
                }
                if region_data[offset + HASH_OFFSET + 3] != 0xFF {
                    self.read_buffer.replace(Some(region_data));
                    return Err(ErrorCode::CorruptData);
                }
                if region_data[offset + HASH_OFFSET + 4] != 0xFF {
                    self.read_buffer.replace(Some(region_data));
                    return Err(ErrorCode::CorruptData);
                }
                if region_data[offset + HASH_OFFSET + 5] != 0xFF {
                    self.read_buffer.replace(Some(region_data));
                    return Err(ErrorCode::CorruptData);
                }
                if region_data[offset + HASH_OFFSET + 6] != 0xFF {
                    self.read_buffer.replace(Some(region_data));
                    return Err(ErrorCode::CorruptData);
                }
                if region_data[offset + HASH_OFFSET + 7] != 0xFF {
                    self.read_buffer.replace(Some(region_data));
                    return Err(ErrorCode::CorruptData);
                }

                // If we get here we have found an empty spot

                // Copy in new header
                // This is a little painful, but avoids any unsafe Rust
                region_data[offset + VERSION_OFFSET] = header.version;
                region_data[offset + LEN_OFFSET] =
                    (header.len >> 8) as u8 & 0x0F | (header.flags << 4) & 0xF0;
                region_data[offset + LEN_OFFSET + 1] = (header.len & 0xFF) as u8;
                region_data[offset + HASH_OFFSET] = (header.hashed_key >> 56) as u8;
                region_data[offset + HASH_OFFSET + 1] = (header.hashed_key >> 48) as u8;
                region_data[offset + HASH_OFFSET + 2] = (header.hashed_key >> 40) as u8;
                region_data[offset + HASH_OFFSET + 3] = (header.hashed_key >> 32) as u8;
                region_data[offset + HASH_OFFSET + 4] = (header.hashed_key >> 24) as u8;
                region_data[offset + HASH_OFFSET + 5] = (header.hashed_key >> 16) as u8;
                region_data[offset + HASH_OFFSET + 6] = (header.hashed_key >> 8) as u8;
                region_data[offset + HASH_OFFSET + 7] = (header.hashed_key) as u8;

                // Hash the new header data
                check_sum.update(&region_data[offset + VERSION_OFFSET..=offset + HASH_OFFSET + 7]);

                // Copy the value
                let slice = &mut region_data[(offset + HEADER_LENGTH)..(offset + package_length)];
                slice.copy_from_slice(value);

                // Include the value in the hash
                check_sum.update(value);

                // Append a Check Hash
                let check_sum = check_sum.finalise();
                let slice = &mut region_data
                    [(offset + package_length)..(offset + package_length + CHECK_SUM_LEN)];
                slice.copy_from_slice(&check_sum.to_ne_bytes());

                // Write the data back to the region
                if let Err(e) = self.controller.write(
                    S * new_region as usize + offset,
                    &region_data[offset..(offset + package_length + CHECK_SUM_LEN)],
                ) {
                    self.read_buffer.replace(Some(region_data));
                    match e {
                        ErrorCode::WriteNotReady(_) => return Ok(SuccessCode::Queued),
                        _ => return Err(e),
                    }
                }

                self.read_buffer.replace(Some(region_data));
                return Ok(SuccessCode::Written);
            }
        }
    }

    /// Retrieves the value from flash storage.
    ///
    /// `hash`: A hashed key.
    /// `buf`: A buffer to store the value to.
    ///
    /// On success nothing will be returned.
    /// On error a `ErrorCode` will be returned.
    ///
    /// If a power loss occurs before success is returned the data is
    /// assumed to be lost.
    pub fn get_key(&self, hash: u64, buf: &mut [u8]) -> Result<SuccessCode, ErrorCode> {
        let region = self.get_region(hash);

        let mut region_offset: isize = 0;

        loop {
            let crc = crc32::Crc::new();
            let mut check_sum = crc.digest();
            let new_region = match self.state.get() {
                State::None => region as isize + region_offset,
                State::Init(state) => {
                    match state {
                        InitState::GetKeyReadRegion(reg) => reg as isize,
                        _ => {
                            // Get the data from that region
                            region as isize + region_offset
                        }
                    }
                }
                State::GetKey(key_state) => match key_state {
                    KeyState::ReadRegion(reg) => reg as isize,
                },
                _ => unreachable!(),
            };

            // Get the data from that region
            let mut region_data = self.read_buffer.take().unwrap();
            if self.state.get() != State::GetKey(KeyState::ReadRegion(new_region as usize))
                && self.state.get() != State::Init(InitState::GetKeyReadRegion(new_region as usize))
            {
                match self
                    .controller
                    .read_region(new_region as usize, 0, &mut region_data)
                {
                    Ok(()) => {}
                    Err(e) => {
                        self.read_buffer.replace(Some(region_data));
                        if let ErrorCode::ReadNotReady(reg) = e {
                            self.state.set(State::GetKey(KeyState::ReadRegion(reg)));
                        }
                        return Err(e);
                    }
                };
            }

            match self.find_key_offset(hash, region_data) {
                Ok((offset, total_length)) => {
                    // Add the header data to the check hash
                    check_sum.update(&region_data[offset..(HEADER_LENGTH + offset)]);

                    // Make sure if will fit in the buffer
                    if buf.len() < (total_length as usize - HEADER_LENGTH - CHECK_SUM_LEN) {
                        self.read_buffer.replace(Some(region_data));
                        return Err(ErrorCode::BufferTooSmall(
                            total_length as usize - HEADER_LENGTH - CHECK_SUM_LEN,
                        ));
                    }

                    // Copy in the value
                    for i in 0..(total_length as usize - HEADER_LENGTH - CHECK_SUM_LEN) {
                        buf[i] = region_data[offset + HEADER_LENGTH + i];
                        check_sum.update(&[buf[i]])
                    }

                    // Check the hash
                    let check_sum = check_sum.finalise();
                    let check_sum = check_sum.to_ne_bytes();

                    if check_sum[3] != region_data[offset + total_length as usize - 1]
                        || check_sum[2] != region_data[offset + total_length as usize - 2]
                        || check_sum[1] != region_data[offset + total_length as usize - 3]
                        || check_sum[0] != region_data[offset + total_length as usize - 4]
                    {
                        self.read_buffer.replace(Some(region_data));
                        return Err(ErrorCode::InvalidCheckSum);
                    }

                    self.read_buffer.replace(Some(region_data));
                    return Ok(SuccessCode::Complete);
                }
                Err((cont, e)) => {
                    self.read_buffer.replace(Some(region_data));

                    if cont {
                        match self.increment_region_offset(new_region) {
                            Some(o) => {
                                region_offset = o;
                            }
                            None => {
                                return Err(e);
                            }
                        }
                    } else {
                        return Err(e);
                    }
                }
            }
        }
    }

    /// Invalidates the key in flash storage
    ///
    /// `hash`: A hashed key.
    ///
    /// On success nothing will be returned.
    /// On error a `ErrorCode` will be returned.
    ///
    /// If a power loss occurs before success is returned the data is
    /// assumed to be lost.
    pub fn invalidate_key(&self, hash: u64) -> Result<SuccessCode, ErrorCode> {
        let region = self.get_region(hash);

        let mut region_offset: isize = 0;

        loop {
            // Get the data from that region
            let new_region = match self.state.get() {
                State::None => region as isize + region_offset,
                State::InvalidateKey(key_state) => match key_state {
                    KeyState::ReadRegion(reg) => reg as isize,
                },
                _ => unreachable!(),
            };

            // Get the data from that region
            let mut region_data = self.read_buffer.take().unwrap();
            if self.state.get() != State::InvalidateKey(KeyState::ReadRegion(new_region as usize)) {
                match self
                    .controller
                    .read_region(new_region as usize, 0, &mut region_data)
                {
                    Ok(()) => {}
                    Err(e) => {
                        self.read_buffer.replace(Some(region_data));
                        if let ErrorCode::ReadNotReady(reg) = e {
                            self.state
                                .set(State::InvalidateKey(KeyState::ReadRegion(reg)));
                        }
                        return Err(e);
                    }
                };
            }

            match self.find_key_offset(hash, region_data) {
                Ok((offset, _data_len)) => {
                    // We found a key, let's delete it
                    region_data[offset + LEN_OFFSET] &= !0x80;

                    if let Err(e) = self.controller.write(
                        S * new_region as usize + offset + LEN_OFFSET,
                        &region_data[offset + LEN_OFFSET..offset + LEN_OFFSET + 1],
                    ) {
                        self.read_buffer.replace(Some(region_data));
                        match e {
                            ErrorCode::WriteNotReady(_) => return Ok(SuccessCode::Queued),
                            _ => return Err(e),
                        }
                    }

                    self.read_buffer.replace(Some(region_data));
                    return Ok(SuccessCode::Written);
                }
                Err((cont, e)) => {
                    self.read_buffer.replace(Some(region_data));

                    if cont {
                        match self.increment_region_offset(new_region) {
                            Some(o) => {
                                region_offset = o;
                            }
                            None => {
                                return Err(e);
                            }
                        }
                    } else {
                        return Err(e);
                    }
                }
            }
        }
    }

    fn garbage_collect_region(&self, region: usize) -> Result<usize, ErrorCode> {
        // Get the data from that region
        let mut region_data = self.read_buffer.take().unwrap();
        if self.state.get() != State::GarbageCollect(RubbishState::ReadRegion(region)) {
            match self.controller.read_region(region, 0, &mut region_data) {
                Ok(()) => {}
                Err(e) => {
                    self.read_buffer.replace(Some(region_data));
                    if let ErrorCode::ReadNotReady(reg) = e {
                        self.state
                            .set(State::GarbageCollect(RubbishState::ReadRegion(reg)));
                    }
                    return Err(e);
                }
            };
        }

        let mut entry_found = false;
        let mut offset: usize = 0;

        loop {
            if offset >= S {
                // We have reached the end of the region without finding a
                // valid object. All entries must be marked for deletion then.
                break;
            }

            // Check to see if we have data
            if region_data[offset + VERSION_OFFSET] != 0xFF {
                // We found a version, check that we support it
                if region_data[offset + VERSION_OFFSET] != VERSION {
                    self.read_buffer.replace(Some(region_data));
                    return Err(ErrorCode::UnsupportedVersion);
                }

                entry_found = true;

                // Find this entries length
                let total_length = ((region_data[offset + LEN_OFFSET] as u16) & !0xF0) << 8
                    | region_data[offset + LEN_OFFSET + 1] as u16;

                // Check to see if the entry has been deleted
                if region_data[offset + LEN_OFFSET] & 0x80 != 0x80 {
                    // The entry has been deleted, this region might be ready
                    // for erasure.
                    // Increment our offset by the length and repeat the loop
                    offset += total_length as usize;
                    continue;
                }

                // We have found a valid entry!
                // Don't perform an erase!
                self.read_buffer.replace(Some(region_data));
                return Ok(0);
            } else {
                // We hit the end of valid data.
                // The possible outcomes:
                //    * The region is empty, we don't need to do anything
                //    * The region has entries, all of which are marked for
                //      deletion
                if !entry_found {
                    // We didn't find anything, don't bother erasing an empty region.
                    self.read_buffer.replace(Some(region_data));
                    return Ok(0);
                }
                break;
            }
        }

        self.read_buffer.replace(Some(region_data));

        // If we got down here, the region is ready to be erased.

        if let Err(e) = self.controller.erase_region(region) {
            if let ErrorCode::EraseNotReady(reg) = e {
                self.state
                    .set(State::GarbageCollect(RubbishState::EraseRegion(reg)));
            }
            return Err(e);
        }

        Ok(S)
    }

    /// Perform a garbage collection on TicKV
    ///
    /// On success the number of bytes freed will be returned.
    /// On error a `ErrorCode` will be returned.
    pub fn garbage_collect(&self) -> Result<usize, ErrorCode> {
        let num_region = self.flash_size / S;
        let mut flash_freed = 0;
        let start = match self.state.get() {
            State::None => 0,
            State::GarbageCollect(state) => match state {
                RubbishState::ReadRegion(reg) => reg,
                // We already erased region reg, so move to the next one
                RubbishState::EraseRegion(reg) => reg + 1,
            },
            _ => unreachable!(),
        };

        for i in start..num_region {
            match self.garbage_collect_region(i) {
                Ok(freed) => flash_freed += freed,
                Err(e) => return Err(e),
            }
        }

        Ok(flash_freed)
    }
}
