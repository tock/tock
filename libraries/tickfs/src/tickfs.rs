//! The TickFS implementation.

use crate::error_codes::ErrorCode;
use crate::flash_controller::FlashController;
use core::cell::Cell;
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;

/// The current version of TickFS
pub const VERSION: u8 = 0;

#[derive(Debug)]
/// The operation to try and complete.
/// This is used when returning from a complete async `FlashController` call.
pub enum Operation {
    /// The `append_key()` function
    AppendKey,
    /// The `get_key()` function
    GetKey,
    /// The `invalidate_key()` function
    InvalidateKey,
}

#[derive(Debug, Clone, Copy)]
/// The current state machine when trying to complete a previous operation.
/// This is used when returning from a complete async `FlashController` call.
pub enum State {
    /// No previous state
    None,
    /// The requested read has completed, including the region that was read
    ReadComplete(isize),
    /// The requested erase has completed, including the region that was read
    EraseComplete(usize),
}

/// The struct storing all of the TickFS information.
pub struct TickFS<'a, C: FlashController, H: Hasher> {
    controller: C,
    flash_size: usize,
    region_size: usize,
    read_buffer: Cell<&'a mut [u8]>,
    phantom_hasher: PhantomData<H>,
    continue_state: Cell<State>,
}

/// This is the current object header used for TickFS objects
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
pub(crate) const CHECK_SUM_LEN: usize = 8;

const MAIN_KEY: &[u8; 16] = b"tickfs-super-key";

/// This is the main TickFS struct.
impl<'a, C: FlashController, H: Hasher> TickFS<'a, C, H> {
    /// Create a new struct
    ///
    /// `C`: An implementation of the `FlashController` trait
    ///
    /// `controller`: An new struct implementing `FlashController`
    /// `flash_size`: The total size of the flash used for TickFS
    /// `region_size`: The smallest size that can be erased in a single operation.
    ///                This must be a multiple of the start address and `flash_size`
    pub fn new(
        controller: C,
        read_buffer: &'a mut [u8],
        flash_size: usize,
        region_size: usize,
    ) -> Self {
        Self {
            controller,
            flash_size,
            region_size,
            read_buffer: Cell::new(read_buffer),
            phantom_hasher: PhantomData,
            continue_state: Cell::new(State::None),
        }
    }

    /// This function setups the flash region to be used as a key-value store.
    /// If the region is already initalised this won't make any changes.
    ///
    /// `H`: An implementation of a `core::hash::Hasher` trait. This MUST
    ///      always return the same hash for the same input. That is the
    ///      implementation can NOT change over time.
    ///
    /// If the specified region has not already been setup for TickFS
    /// the entire region will be erased.
    ///
    /// On success nothing will be returned.
    /// On error a `ErrorCode` will be returned.
    pub fn initalise(&self, hash_function: (&mut H, &mut H)) -> Result<(), ErrorCode> {
        let mut buf: [u8; 0] = [0; 0];

        match self.get_key(hash_function.0, MAIN_KEY, &mut buf) {
            Ok(()) => Ok(()),
            Err(e) => {
                match e {
                    ErrorCode::ReadNotReady(reg) => Err(ErrorCode::ReadNotReady(reg)),
                    _ => {
                        // Erase all regions
                        let mut start = 0;
                        if let State::EraseComplete(reg) = self.continue_state.get() {
                            start = reg;
                        }

                        if start < (self.flash_size / self.region_size) {
                            for r in start..(self.flash_size / self.region_size) {
                                self.controller.erase_region(r)?
                            }
                        }

                        // Save the main key
                        self.append_key(hash_function.1, MAIN_KEY, &buf)
                    }
                }
            }
        }
    }

    /// Continue the initalise process after an async access.
    ///
    /// `state`: The current state of the previous operation.
    /// `H`: An implementation of a `core::hash::Hasher` trait. This MUST
    ///      always return the same hash for the same input. That is the
    ///      implementation can NOT change over time.
    ///
    /// On success nothing will be returned.
    /// On error a `ErrorCode` will be returned.
    pub fn continue_initalise(
        &self,
        state: State,
        hash_function: (&mut H, &mut H),
    ) -> Result<(), ErrorCode> {
        self.continue_state.set(state);
        self.initalise(hash_function)
    }

    /// Generate the hash and region number from a key
    fn get_hash_and_region(&self, hash_function: &mut H, key: &[u8]) -> (u64, usize) {
        // Generate a hash of the key
        key.hash(hash_function);
        let hash = hash_function.finish();

        assert_ne!(hash, 0xFFFF_FFFF_FFFF_FFFF);
        assert_ne!(hash, 0);

        // Determine the number of regions
        let num_region = self.flash_size / self.region_size;

        // Determine the block where the data should be
        let region = (hash as usize & 0xFFFF) % num_region;

        (hash, region)
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
            if new_offset as usize > ((self.flash_size / self.region_size) - 1) {
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
            if offset + HEADER_LENGTH >= self.region_size {
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

    /// Continue a previous key operation after an async access.
    ///
    /// `operation`: The previous operation that should be continued.
    /// `state`: The current state of the previous operation.
    /// `hash_function`: Hash function with no previous state. This is
    ///                  usually a newly created hash.
    /// `key`: A unhashed key. This will be hashed internally. This key
    ///        will be used in future to retrieve or remove the `value`.
    /// `value`: A buffer containing the data to be stored to flash.
    /// `buf`: A buffer to store the value to.
    ///
    /// On success nothing will be returned.
    /// On error a `ErrorCode` will be returned.
    pub fn continue_operation(
        &self,
        operation: Operation,
        state: State,
        hash_function: Option<&mut H>,
        key: Option<&[u8]>,
        value: Option<&[u8]>,
        buf: Option<&mut [u8]>,
    ) -> Result<(), ErrorCode> {
        self.continue_state.set(state);
        match operation {
            Operation::AppendKey => {
                self.append_key(hash_function.unwrap(), key.unwrap(), value.unwrap())
            }
            Operation::GetKey => self.get_key(hash_function.unwrap(), key.unwrap(), buf.unwrap()),
            Operation::InvalidateKey => self.invalidate_key(hash_function.unwrap(), key.unwrap()),
        }
    }

    /// Appends the key/value pair to flash storage.
    ///
    /// `hash_function`: Hash function with no previous state. This is
    ///                  usually a newly created hash.
    /// `key`: A unhashed key. This will be hashed internally. This key
    ///        will be used in future to retrieve or remove the `value`.
    /// `value`: A buffer containing the data to be stored to flash.
    ///
    /// On success nothing will be returned.
    /// On error a `ErrorCode` will be returned.
    pub fn append_key(
        &self,
        hash_function: &mut H,
        key: &[u8],
        value: &[u8],
    ) -> Result<(), ErrorCode> {
        let (hash, region) = self.get_hash_and_region(hash_function, key);

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
            let new_region = match self.continue_state.get() {
                State::ReadComplete(reg) => reg,
                _ => {
                    // Get the data from that region
                    region as isize + region_offset
                }
            };

            let mut region_data = self.read_buffer.take();
            match self
                .controller
                .read_region(new_region as usize, 0, &mut region_data)
            {
                Ok(()) => {}
                Err(e) => {
                    self.read_buffer.replace(region_data);
                    return Err(e);
                }
            };

            if self.find_key_offset(hash, &region_data).is_ok() {
                // Check to make sure we don't already have this key
                self.read_buffer.replace(region_data);
                return Err(ErrorCode::KeyAlreadyExists);
            }

            let mut offset: usize = 0;

            loop {
                if offset + package_length >= self.region_size {
                    // We have reached the end of the region
                    // We will need to try the next region

                    // Replace the buffer
                    self.read_buffer.replace(region_data);

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
                        self.read_buffer.replace(region_data);
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
                    self.read_buffer.replace(region_data);
                    return Err(ErrorCode::CorruptData);
                }
                if region_data[offset + HASH_OFFSET + 1] != 0xFF {
                    self.read_buffer.replace(region_data);
                    return Err(ErrorCode::CorruptData);
                }
                if region_data[offset + HASH_OFFSET + 2] != 0xFF {
                    self.read_buffer.replace(region_data);
                    return Err(ErrorCode::CorruptData);
                }
                if region_data[offset + HASH_OFFSET + 3] != 0xFF {
                    self.read_buffer.replace(region_data);
                    return Err(ErrorCode::CorruptData);
                }
                if region_data[offset + HASH_OFFSET + 4] != 0xFF {
                    self.read_buffer.replace(region_data);
                    return Err(ErrorCode::CorruptData);
                }
                if region_data[offset + HASH_OFFSET + 5] != 0xFF {
                    self.read_buffer.replace(region_data);
                    return Err(ErrorCode::CorruptData);
                }
                if region_data[offset + HASH_OFFSET + 6] != 0xFF {
                    self.read_buffer.replace(region_data);
                    return Err(ErrorCode::CorruptData);
                }
                if region_data[offset + HASH_OFFSET + 7] != 0xFF {
                    self.read_buffer.replace(region_data);
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
                for d in &region_data[offset + VERSION_OFFSET..=offset + HASH_OFFSET + 7] {
                    hash_function.write_u8(*d);
                }

                // Copy the value
                let slice = &mut region_data[(offset + HEADER_LENGTH)..(offset + package_length)];
                slice.copy_from_slice(value);

                // Include the value in the hash
                value.hash(hash_function);

                // Append a Check Hash
                let check_sum = hash_function.finish();
                let slice = &mut region_data
                    [(offset + package_length)..(offset + package_length + CHECK_SUM_LEN)];
                slice.copy_from_slice(&check_sum.to_ne_bytes());

                // Write the data back to the region
                if let Err(e) = self.controller.write(
                    self.region_size * new_region as usize + offset,
                    &region_data[offset..(offset + package_length + CHECK_SUM_LEN)],
                ) {
                    self.read_buffer.replace(region_data);
                    return Err(e);
                }

                self.read_buffer.replace(region_data);
                return Ok(());
            }
        }
    }

    /// Retrieves the value from flash storage.
    ///
    /// `hash_function`: Hash function with no previous state. This is
    ///                  usually a newly created hash.
    /// `key`: A unhashed key. This will be hashed internally.
    /// `buf`: A buffer to store the value to.
    ///
    /// On success nothing will be returned.
    /// On error a `ErrorCode` will be returned.
    ///
    /// If a power loss occurs before success is returned the data is
    /// assumed to be lost.
    pub fn get_key(
        &self,
        hash_function: &mut H,
        key: &[u8],
        buf: &mut [u8],
    ) -> Result<(), ErrorCode> {
        let (hash, region) = self.get_hash_and_region(hash_function, key);

        let mut region_offset: isize = 0;

        loop {
            let new_region = match self.continue_state.get() {
                State::ReadComplete(reg) => reg,
                _ => {
                    // Get the data from that region
                    region as isize + region_offset
                }
            };

            // Get the data from that region
            let mut region_data = self.read_buffer.take();
            match self
                .controller
                .read_region(new_region as usize, 0, &mut region_data)
            {
                Ok(()) => {}
                Err(e) => {
                    self.read_buffer.replace(region_data);
                    return Err(e);
                }
            };

            match self.find_key_offset(hash, &region_data) {
                Ok((offset, total_length)) => {
                    // Add the header data to the check hash
                    for i in 0..HEADER_LENGTH {
                        hash_function.write_u8(region_data[offset + i]);
                    }

                    // Make sure if will fit in the buffer
                    if buf.len() < (total_length as usize - HEADER_LENGTH - CHECK_SUM_LEN) {
                        self.read_buffer.replace(region_data);
                        return Err(ErrorCode::BufferTooSmall(
                            total_length as usize - HEADER_LENGTH - CHECK_SUM_LEN,
                        ));
                    }

                    // Copy in the value
                    for i in 0..(total_length as usize - HEADER_LENGTH - CHECK_SUM_LEN) {
                        buf[i] = region_data[offset + HEADER_LENGTH + i];
                    }

                    // Include the value in the hash
                    buf.hash(hash_function);

                    // Check the hash
                    let check_sum = hash_function.finish();

                    let check_sum = check_sum.to_ne_bytes();

                    if check_sum[7] != region_data[offset + total_length as usize - 1]
                        || check_sum[6] != region_data[offset + total_length as usize - 2]
                        || check_sum[5] != region_data[offset + total_length as usize - 3]
                        || check_sum[4] != region_data[offset + total_length as usize - 4]
                        || check_sum[3] != region_data[offset + total_length as usize - 5]
                        || check_sum[2] != region_data[offset + total_length as usize - 6]
                        || check_sum[1] != region_data[offset + total_length as usize - 7]
                        || check_sum[0] != region_data[offset + total_length as usize - 8]
                    {
                        self.read_buffer.replace(region_data);
                        return Err(ErrorCode::InvalidCheckSum);
                    }

                    self.read_buffer.replace(region_data);
                    return Ok(());
                }
                Err((cont, e)) => {
                    self.read_buffer.replace(region_data);

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
    /// `hash_function`: Hash function with no previous state. This is
    ///                  usually a newly created hash.
    /// `key`: A unhashed key. This will be hashed internally.
    ///
    /// On success nothing will be returned.
    /// On error a `ErrorCode` will be returned.
    ///
    /// If a power loss occurs before success is returned the data is
    /// assumed to be lost.
    pub fn invalidate_key(&self, hash_function: &mut H, key: &[u8]) -> Result<(), ErrorCode> {
        let (hash, region) = self.get_hash_and_region(hash_function, key);

        let mut region_offset: isize = 0;

        loop {
            // Get the data from that region
            let new_region = match self.continue_state.get() {
                State::ReadComplete(reg) => reg,
                _ => {
                    // Get the data from that region
                    region as isize + region_offset
                }
            };

            // Get the data from that region
            let mut region_data = self.read_buffer.take();
            match self
                .controller
                .read_region(new_region as usize, 0, &mut region_data)
            {
                Ok(()) => {}
                Err(e) => {
                    self.read_buffer.replace(region_data);
                    return Err(e);
                }
            };

            match self.find_key_offset(hash, &region_data) {
                Ok((offset, _data_len)) => {
                    // We found a key, let's delete it
                    region_data[offset + LEN_OFFSET] &= !0x80;

                    if let Err(e) = self.controller.write(
                        self.region_size * new_region as usize + offset + LEN_OFFSET,
                        &region_data[offset + LEN_OFFSET..offset + LEN_OFFSET + 1],
                    ) {
                        self.read_buffer.replace(region_data);
                        return Err(e);
                    }

                    self.read_buffer.replace(region_data);
                    return Ok(());
                }
                Err((cont, e)) => {
                    self.read_buffer.replace(region_data);

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
        let mut region_data = self.read_buffer.take();
        match self.controller.read_region(region, 0, &mut region_data) {
            Ok(()) => {}
            Err(e) => {
                self.read_buffer.replace(region_data);
                return Err(e);
            }
        };

        let mut entry_found = false;
        let mut offset: usize = 0;

        loop {
            if offset >= self.region_size {
                // We have reached the end of the region without finding a
                // valid object. All entries must be marked for deletion then.
                break;
            }

            // Check to see if we have data
            if region_data[offset + VERSION_OFFSET] != 0xFF {
                // We found a version, check that we support it
                if region_data[offset + VERSION_OFFSET] != VERSION {
                    self.read_buffer.replace(region_data);
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
                self.read_buffer.replace(region_data);
                return Ok(0);
            } else {
                // We hit the end of valid data.
                // The possible outcomes:
                //    * The region is empty, we don't need to do anything
                //    * The region has entries, all of which are marked for
                //      deletion
                if !entry_found {
                    // We didn't find anything, don't bother erasing an empty region.
                    self.read_buffer.replace(region_data);
                    return Ok(0);
                }
                break;
            }
        }

        self.read_buffer.replace(region_data);

        // If we got down here, the region is ready to be erased.

        if let Err(e) = self.controller.erase_region(region) {
            return Err(e);
        }

        Ok(self.region_size)
    }

    /// Perform a garbage collection on TickFS
    ///
    /// On success the number of bytes freed will be returned.
    /// On error a `ErrorCode` will be returned.
    pub fn garbage_collect(&self) -> Result<usize, ErrorCode> {
        let num_region = self.flash_size / self.region_size;
        let mut flash_freed = 0;
        let start = match self.continue_state.get() {
            State::ReadComplete(reg) => reg as usize,
            State::EraseComplete(reg) => reg,
            _ => 0,
        };

        for i in start..num_region {
            match self.garbage_collect_region(i) {
                Ok(freed) => flash_freed += freed,
                Err(e) => return Err(e),
            }
        }

        Ok(flash_freed)
    }

    /// Continue the garbage collection process after an async access.
    ///
    /// `state`: The current state of the previous operation.
    ///
    /// On success the number of bytes freed will be returned.
    /// On error a `ErrorCode` will be returned.
    pub fn continue_garbage_collection(&self, state: State) -> Result<usize, ErrorCode> {
        self.continue_state.set(state);
        self.garbage_collect()
    }
}
