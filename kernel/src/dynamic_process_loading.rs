// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Dynamic Process Loader for application loading and updating at runtime.
//!
//! These functions facilitate dynamic application loading and process creation
//! during runtime without requiring the user to restart the device.

use core::cell::Cell;

use crate::config;
use crate::debug;
use crate::hil::nonvolatile_storage::{NonvolatileStorage, NonvolatileStorageClient};
use crate::process::{self, ProcessLoadingAsync, ProcessLoadingAsyncClient};
use crate::process_binary::ProcessBinaryError;
use crate::process_loading::ProcessLoadError;
use crate::utilities::cells::{MapCell, OptionalCell, TakeCell};
use crate::utilities::leasable_buffer::SubSliceMut;
use crate::ErrorCode;

// Fixed max supported process slots to store the start addresses of processes
// to write padding.
const MAX_PROCS: usize = 10;

/// Expected buffer length for storing application binaries.
pub const BUF_LEN: usize = 512;

/// The number of bytes in the TBF header for a padding app.
const PADDING_TBF_HEADER_LENGTH: usize = 16;

#[derive(Clone, Copy, PartialEq)]
pub enum State {
    Idle,
    Setup,
    AppWrite,
    Load,
    PaddingWrite,
    Fail,
}

/// Whether a new app needs a padding app inserted before and/or after the newly
/// stored app.
#[derive(Clone, Copy, PartialEq, Default)]
pub enum PaddingRequirement {
    #[default]
    None,
    PrePad,
    PostPad,
    PreAndPostPad,
}

/// What is stored in flash at a particular address.
#[derive(PartialEq)]
enum StoredInFlash {
    /// There is an app of `size` bytes.
    ValidApp(usize),
    /// There is a padding app.
    PaddingApp,
    /// There is no app.
    Empty,
}

/// Addresses of where the new process will be stored.
#[derive(Clone, Copy, Default)]
pub struct ProcessLoadMetadata {
    pub new_app_start_addr: usize,
    pub new_app_length: usize,
    pub previous_app_end_addr: usize,
    pub next_app_start_addr: usize,
    pub padding_requirement: PaddingRequirement,
}

/// This interface supports loading processes at runtime.
pub trait DynamicProcessLoading {
    /// Call to request loading a new process.
    ///
    /// This informs the kernel we want to load a process and the size of the
    /// entire process binary. The kernel will try to find a suitable location
    /// in flash to store said process.
    ///
    /// Return value:
    /// - `Ok((length, wait_for_setup))`: If there is a place to load the
    ///   process, the function will return `Ok()` with the size of the region
    ///   to store the process, and whether the process loader is waiting to set
    ///   up. This usually happens when we have to write a post pad app. The
    ///   client app is unable to write new app data until the process loader
    ///   finishes writing the padding app. So if tihs flag is set, then the
    ///   client app has to wait until the setup_done subscribe callback is
    ///   received.
    /// - `Err(ErrorCode)`: If there is nowhere to store the process a suitable
    ///   `ErrorCode` will be returned.
    fn setup(&self, app_length: usize) -> Result<(usize, bool), ErrorCode>;

    /// Instruct the kernel to write data to the flash.
    ///
    /// `offset` is where to start writing within the region allocated
    /// for the new process binary from the `setup()` call.
    ///
    /// The caller must write the first 8 bytes of the process with valid header
    /// data. Writes must either be after the first 8 bytes or include the
    /// entire first 8 bytes.
    ///
    /// Returns an error if the write is outside of the permitted region or is
    /// writing an invalid header.
    fn write_app_data(
        &self,
        buffer: SubSliceMut<'static, u8>,
        offset: usize,
    ) -> Result<(), ErrorCode>;

    /// Instruct the kernel to write data to the flash.
    ///
    /// `offset` is where to start writing within the region allocated for the
    /// new process binary from the `setup()` call.
    ///
    /// The caller must write the first 8 bytes of the process with valid header
    /// data. Writes must either be after the first 8 bytes or include the
    /// entire first 8 bytes.
    ///
    /// Returns an error if the write is outside of the permitted region or is
    /// writing an invalid header.
    fn load(&self) -> Result<(), ErrorCode>;

    /// Sets a client for the DynamicProcessLoading Object
    ///
    /// When the client operation is done, it calls the `write_app_data_done()`
    /// function.
    fn set_client(&self, client: &'static dyn DynamicProcessLoadingClient);
}

/// The callback for dynamic process loading.
pub trait DynamicProcessLoadingClient {
    /// Any setup work is done and we are ready to load the process binary.
    fn setup_done(&self);

    /// The provided app binary buffer has been stored.
    fn write_app_data_done(&self, buffer: &'static mut [u8], length: usize);

    /// The new app has been loaded.
    fn load_done(&self);
}

/// Dynamic process loading machine.
pub struct DynamicProcessLoader<'a> {
    procs: MapCell<&'static mut [Option<&'static dyn process::Process>]>,
    flash: Cell<&'static [u8]>,
    new_process_flash: OptionalCell<&'static [u8]>,
    flash_driver: &'a dyn NonvolatileStorage<'a>,
    loader_driver: &'a dyn ProcessLoadingAsync<'a>,
    buffer: TakeCell<'static, [u8]>,
    client: OptionalCell<&'static dyn DynamicProcessLoadingClient>,
    process_load_metadata: OptionalCell<ProcessLoadMetadata>,
    state: Cell<State>,
}

impl<'a> DynamicProcessLoader<'a> {
    pub fn new(
        processes: &'static mut [Option<&'static dyn process::Process>],
        flash: &'static [u8],
        flash_driver: &'a dyn NonvolatileStorage<'a>,
        loader_driver: &'a dyn ProcessLoadingAsync<'a>,
        buffer: &'static mut [u8],
    ) -> Self {
        Self {
            procs: MapCell::new(processes),
            flash: Cell::new(flash),
            new_process_flash: OptionalCell::empty(),
            flash_driver: flash_driver,
            loader_driver: loader_driver,
            buffer: TakeCell::new(buffer),
            client: OptionalCell::empty(),
            process_load_metadata: OptionalCell::empty(),
            state: Cell::new(State::Idle),
        }
    }

    /// Function to find the next available slot in the processes array.
    fn find_open_process_slot(&self) -> Option<usize> {
        self.procs.map_or(None, |procs| {
            for (i, p) in procs.iter().enumerate() {
                if p.is_none() {
                    return Some(i);
                }
            }
            None
        })
    }

    /// Function to reset variables and states.
    fn reset_process_loading_metadata(&self) {
        self.state.set(State::Idle);
        self.process_load_metadata.take();
    }

    /// This function checks whether the new app will fit in the bounds dictated
    /// by the start address and length provided during the setup phase. This
    /// function then also computes where in flash the data should be written
    /// based on whether the call is coming during the app writing phase, or the
    /// padding phase.
    ///
    /// This function returns the physical address in flash where the write is
    /// supposed to happen.
    fn compute_address(&self, offset: usize, length: usize) -> Result<usize, ErrorCode> {
        let mut new_app_len: usize = 0;
        let mut new_app_addr: usize = 0;
        if let Some(metadata) = self.process_load_metadata.get() {
            new_app_addr = metadata.new_app_start_addr;
            new_app_len = metadata.new_app_length;
        }

        match self.state.get() {
            State::AppWrite => {
                // Check if there is an overflow while adding length and offset.
                match offset.checked_add(length) {
                    Some(result) => {
                        // Check if the length of the new write block goes over
                        // the total size alloted to the new application. We
                        // also check if the new app is trying to write beyond
                        // the bounds of the flash region allocated to it.
                        if length > new_app_len || result > new_app_len {
                            // This means the app is out of bounds.
                            Err(ErrorCode::INVAL)
                        } else {
                            // We compute the new address to write the app
                            // binary segment.
                            Ok(offset + new_app_addr)
                        }
                    }
                    None => Err(ErrorCode::INVAL),
                }
            }
            // If we are going to write the padding header, we already know
            // where to write in flash, so we don't have to add the start
            // address
            State::Setup | State::Load | State::PaddingWrite => Ok(offset),
            // We aren't supposed to be able to write unless we are in one of
            // the first two write states
            _ => Err(ErrorCode::FAIL),
        }
    }

    /// Compute the physical address where we should write the data and then
    /// write it.
    ///
    /// Current limitation: There are two limitations with the current
    /// implementation of how tock looks for and loads apps. Assuming the flash
    /// looks like this:
    ///
    /// ```text
    ///  ____________________________________________________
    /// |             |    |            |          |         |
    /// |     App1    | H? |   NewApp?  |   Pad    |   App2  |
    /// |_____________|____|____________|__________|_________|
    /// ```
    ///
    /// Assuming the new app goes in between App 1 and App 2 which are existing
    /// processes, we write the padding after the new app during setup phase.
    ///
    /// - Issue 1: If there is a power cycle as we try to write the header for
    ///   the new app, during the flash erase part of the flash write, we might
    ///   end up with a break in the linked list when the device reboots and we
    ///   never boot app 2.
    ///     - Potential Fix:  Reserve a section of flash to hold an
    ///       index/repository of valid headers of current processes as a fall
    ///       back mechanism in case we end up with corrupt headers.
    /// - Issue 2: If the header is written successfully, but there is a power
    ///   cycle as the rest of the app binary is being written, we could end up
    ///   with the situation where because the header is valid, we could end up
    ///   with memory fragmentation.
    ///     - Potential Fix:  Create a processes monitoring process that is able
    ///       to clean up after corrupt apps and defragment memory?
    fn write(&self, user_buffer: SubSliceMut<'static, u8>, offset: usize) -> Result<(), ErrorCode> {
        let length = user_buffer.len();
        // Take the buffer to perform tbf header validation and write with.
        let buffer = user_buffer.take();

        let physical_address = self.compute_address(offset, length)?;

        // The kernel needs to check if the app is trying to write/overwrite the
        // header. So the app can only write to the first 8 bytes if the app is
        // writing all 8 bytes. Else, the kernel must raise an error. The app is
        // not allowed to write from say, offset 4 because we have to ensure the
        // validity of the header.
        //
        // This means the app is trying to manipulate the space where the TBF
        // header should go. Ideally, we want the app to only write the complete
        // set of 8 bytes which is used to determine if the header is valid. We
        // don't apps to do this, so we return an error.
        if (offset == 0 && length < 8) || (offset != 0 && offset < 8) {
            return Err(ErrorCode::INVAL);
        }

        // Check if we are writing the start of the TBF header.
        //
        // The app is not allowed to manipulate parts of the TBF header, so if
        // it is trying to write at the very beginning of the promised flash
        // region, we require the app writes the entire 8 bytes of the header.
        // This header is then checked for validity.
        if offset == 0 {
            // Pass the first eight bytes of the tbf header to parse out the
            // length of the header and app. We then use those values to see if
            // the app is going to be valid.
            let test_header_slice = buffer.get(0..8).ok_or(ErrorCode::INVAL)?;
            let header = test_header_slice.try_into().or(Err(ErrorCode::FAIL))?;
            let (_version, _header_length, entry_length) =
                match tock_tbf::parse::parse_tbf_header_lengths(header) {
                    Ok((v, hl, el)) => (v, hl, el),
                    Err(tock_tbf::types::InitialTbfParseError::InvalidHeader(_entry_length)) => {
                        // If we have an invalid header, so we return an error
                        return Err(ErrorCode::INVAL);
                    }
                    Err(tock_tbf::types::InitialTbfParseError::UnableToParse) => {
                        // If we could not parse the header, then that's an
                        // issue. We return an Error.
                        return Err(ErrorCode::INVAL);
                    }
                };

            // Check if the length in the header is matching what the app
            // requested during the setup phase also check if the kernel
            // version matches the version indicated in the new application.
            let mut new_app_len = 0;
            if let Some(metadata) = self.process_load_metadata.get() {
                new_app_len = metadata.new_app_length;
            }
            if entry_length as usize != new_app_len {
                return Err(ErrorCode::INVAL);
            }
        }
        self.flash_driver.write(buffer, physical_address, length)
    }

    /// Function to generate the padding header to append after the new app.
    /// This header is created and written to ensure the integrity of the
    /// processes linked list
    fn write_padding_app(&self, padding_app_length: usize, offset: usize) -> Result<(), ErrorCode> {
        // Write the header into the array
        self.buffer.map(|buffer| {
            // First two bytes are the TBF version (2).
            buffer[0] = 2;
            buffer[1] = 0;

            // The next two bytes are the header length (fixed to 16 bytes for
            // padding).
            buffer[2] = (PADDING_TBF_HEADER_LENGTH & 0xff) as u8;
            buffer[3] = ((PADDING_TBF_HEADER_LENGTH >> 8) & 0xff) as u8;

            // The next 4 bytes are the total app length including the header.
            buffer[4] = (padding_app_length & 0xff) as u8;
            buffer[5] = ((padding_app_length >> 8) & 0xff) as u8;
            buffer[6] = ((padding_app_length >> 16) & 0xff) as u8;
            buffer[7] = ((padding_app_length >> 24) & 0xff) as u8;

            // We set the flags to 0.
            for i in 8..12 {
                buffer[i] = 0x00_u8;
            }

            // xor of the previous values
            buffer[12] = buffer[0] ^ buffer[4] ^ buffer[8];
            buffer[13] = buffer[1] ^ buffer[5] ^ buffer[9];
            buffer[14] = buffer[2] ^ buffer[6] ^ buffer[10];
            buffer[15] = buffer[3] ^ buffer[7] ^ buffer[11];
        });

        let flash_end = self.flash.get().as_ptr() as usize + self.flash.get().len();
        self.buffer.take().map_or(Err(ErrorCode::BUSY), |buffer| {
            if flash_end - offset >= PADDING_TBF_HEADER_LENGTH {
                // Write the header only if there are more than 16 bytes.
                // available in the flash.
                let mut padding_slice = SubSliceMut::new(buffer);
                padding_slice.slice(..PADDING_TBF_HEADER_LENGTH);
                // We are only writing the header, so 16 bytes is enough.
                self.write(padding_slice, offset)
            } else {
                // This means we do not have even 16 bytes to write the header.
                Err(ErrorCode::NOMEM)
            }
        })
    }

    /// This function checks if there is a need to pad either before or after
    /// the new app to preserve the linked list.
    ///
    /// When do we pad?
    ///
    /// 1. When there is a process in the processes array which is located in
    ///    flash after the new app but not immediately after, we need to add
    ///    padding between the new app and the existing app.
    /// 2. Due to MPU alignment, the new app may be similarly placed not
    ///    immediately after an existing process, in that case, we need to add
    ///    padding between the previous app and the new app.
    /// 3. If both the above conditions are met, we add both a prepadding and a
    ///    postpadding.
    /// 4. If either of these conditions are not met, we don't pad.
    fn compute_padding_requirement_and_neighbors(
        &self,
        new_app_start_address: usize,
    ) -> (PaddingRequirement, usize, usize) {
        let mut app_length = 0;
        if let Some(metadata) = self.process_load_metadata.get() {
            app_length = metadata.new_app_length;
        }
        // The end address of our newly loaded application.
        let new_app_end_address = new_app_start_address + app_length;
        // To store the address until which we need to write the padding app.
        let mut next_app_start_addr = 0;
        // To store the address from which we need to write the padding app.
        let mut previous_app_end_addr = 0;
        let mut padding_requirement: PaddingRequirement = PaddingRequirement::None;

        let mut processes_start_addresses: [usize; MAX_PROCS] = [0; MAX_PROCS];
        let mut processes_end_addresses: [usize; MAX_PROCS] = [0; MAX_PROCS];

        // Get the start and end addresses in flash of existing processes.
        self.procs.map(|procs| {
            for (procs_index, value) in procs.iter().enumerate() {
                match value {
                    Some(app) => {
                        processes_start_addresses[procs_index] = app.get_addresses().flash_start;
                        processes_end_addresses[procs_index] = app.get_addresses().flash_end;
                    }
                    None => {
                        processes_start_addresses[procs_index] = 0;
                        processes_end_addresses[procs_index] = 0;
                    }
                }
            }
        });

        // We compute the closest neighbor to our app such that:
        //
        // 1. If the new app is placed in between two existing processes, we
        //    compute the closest located processes.
        // 2. Once we compute these values, we determine if we need to write a
        //    pre pad header, or a post pad header, or both.
        // 3. If there are no apps after ours in the process array, we don't do
        //    anything.

        // Postpad requirement.
        if let Some(next_closest_neighbor) = processes_start_addresses
            .iter()
            .filter(|&&x| x > new_app_end_address - 1)
            .min()
        {
            // We found the next closest app in flash.
            next_app_start_addr = *next_closest_neighbor;
            if next_app_start_addr != 0 {
                padding_requirement = PaddingRequirement::PostPad;
            }
        } else {
            if config::CONFIG.debug_load_processes {
                debug!("No App Found after the new app so not adding post padding.");
            }
        }

        // Prepad requirement.
        if let Some(previous_closest_neighbor) = processes_end_addresses
            .iter()
            .filter(|&&x| x < new_app_start_address + 1)
            .max()
        {
            // We found the previous closest app in flash.
            previous_app_end_addr = *previous_closest_neighbor;
            if new_app_start_address - previous_app_end_addr != 0 {
                if padding_requirement == PaddingRequirement::PostPad {
                    padding_requirement = PaddingRequirement::PreAndPostPad;
                } else {
                    padding_requirement = PaddingRequirement::PrePad;
                }
            }
        } else {
            if config::CONFIG.debug_load_processes {
                debug!("No Previous App Found, so not padding before the new app.");
            }
        }
        (
            padding_requirement,
            previous_app_end_addr,
            next_app_start_addr,
        )
    }

    /// Check if there is a padding app at the address.
    fn check_for_app(
        &self,
        possible_app: &'static [u8],
    ) -> Result<StoredInFlash, ProcessBinaryError> {
        // We only need tbf header information to get the size of app which is
        // already loaded.
        let test_header_slice = possible_app
            .get(0..8)
            .ok_or(ProcessBinaryError::NotEnoughFlash)?;

        // Pass the first eight bytes to tbfheader to parse out the length of
        // the tbf header and app. We then use those values to see if we have
        // enough flash remaining to parse the remainder of the header.
        let (version, header_length, _entry_length) =
            match tock_tbf::parse::parse_tbf_header_lengths(
                test_header_slice
                    .try_into()
                    .or(Err(ProcessBinaryError::TbfHeaderNotFound))?,
            ) {
                Ok((v, hl, el)) => (v, hl, el),
                Err(tock_tbf::types::InitialTbfParseError::InvalidHeader(_entry_length)) => {
                    // If we could not parse the header, then we want to skip
                    // over this app and look for the next one.
                    return Err(ProcessBinaryError::TbfHeaderNotFound);
                }
                Err(tock_tbf::types::InitialTbfParseError::UnableToParse) => {
                    // Since Tock apps use a linked list, it is very possible
                    // the header we started to parse is intentionally invalid
                    // to signal the end of apps.
                    return Ok(StoredInFlash::Empty);
                }
            };

        // If a padding app exists at the start address satisfying MPU rules, we
        // load the new app from here!
        let header_flash = possible_app
            .get(0..(header_length as usize))
            .ok_or(ProcessBinaryError::NotEnoughFlash)?;

        let tbf_header = tock_tbf::parse::parse_tbf_header(header_flash, version)?;

        // If this isn't an app (i.e. it is padding).
        if tbf_header.is_app() {
            Ok(StoredInFlash::ValidApp(tbf_header.length() as usize))
        } else {
            Ok(StoredInFlash::PaddingApp)
        }
    }

    /// Check if our new app overlaps with existing apps
    fn check_overlap_region(
        &self,
        new_start_address: usize,
        app_length: usize,
    ) -> Result<(), (usize, ProcessLoadError)> {
        // Find the next open process slot.
        let new_process_count = self.find_open_process_slot().unwrap_or_default();
        let new_process_start_address = new_start_address;
        let new_process_end_address = new_process_start_address + app_length - 1;

        self.procs.map(|procs| {
            for (proc_index, value) in procs.iter().enumerate() {
                if proc_index < new_process_count {
                    let process_start_address = value.unwrap().get_addresses().flash_start;
                    let process_end_address = value.unwrap().get_addresses().flash_end;

                    if new_process_end_address >= process_start_address
                        && new_process_end_address <= process_end_address
                    {
                        /* Case 1
                         *              _________________          _______________           _________________
                         *  ___________|__               |        |              _|_________|__               |
                         * |           |  |              |        |             | |         |  |              |
                         * |   new app |  |  app2        |   or   |   app1      | | new app |  |  app2        |
                         * |___________|__|              |        |             |_|_________|__|              |
                         *             |_________________|        |_______________|         |_________________|
                         *
                         * ^...........^                                           ^........^
                         * In this case, we discard this region and try to find another start address from the end address + 1 of app2
                         */
                        return Err((process_end_address + 1, ProcessLoadError::NotEnoughMemory));
                    } else if new_process_start_address >= process_start_address
                        && new_process_start_address <= process_end_address
                    {
                        /* Case 2
                         *              _________________
                         *  ___________|__               |    _______________
                         * |           |  |              |   |               |
                         * |   app2    |  |  new app     |   |     app3      |
                         * |___________|__|              |   |_______________|
                         *             |_________________|
                         *
                         *                 ^
                         *                 | In this case, the start address of new app is replaced by 'the end address + 1' of app2,
                         *                   and we try to find another start address from the end address + 1 of app2 and recheck for
                         *                   the previous condition
                         */
                        return Err((process_end_address + 1, ProcessLoadError::NotEnoughMemory));
                    }
                }
            }
            Ok(())
        });
        Ok(())
    }

    /// Find where an app of `app_length` can be correctly aligned.
    ///
    /// This currently assumes Cortex-M alignment rules.
    fn next_aligned_address(&self, address: usize, app_length: usize) -> usize {
        let remaining = address % app_length;
        if remaining == 0 {
            address
        } else {
            address + (app_length - remaining)
        }
    }

    /// This function takes in the available flash slice and the app length
    /// specified for the new app, and returns a valid address where the new app
    /// can be flashed such that the linked list and memory alignment rules are
    /// preserved.
    fn find_next_available_address(
        &self,
        flash: &'static [u8],
        app_length: usize,
    ) -> Result<(usize, PaddingRequirement, usize, usize), ErrorCode> {
        let start_address = flash.as_ptr() as usize;
        // We store the address of the new application here.
        let mut new_address = flash.as_ptr() as usize;
        let flash_end = flash.as_ptr() as usize + flash.len() - 1;

        // Iterate through the flash slice looking for a region to place an app
        // that is `app_length` bytes long.
        while new_address < flash_end {
            // Check what is stored at `new_address`.
            let app_type = self
                .check_for_app(
                    flash
                        .get(new_address - start_address..)
                        .ok_or(ErrorCode::NOMEM)?,
                )
                .or(Err(ErrorCode::FAIL))?;

            match app_type {
                StoredInFlash::PaddingApp | StoredInFlash::Empty => {
                    // There is not an app at this address so this can be a
                    // candidate to write the new app.

                    let address_validity_check = self.check_overlap_region(new_address, app_length);

                    match address_validity_check {
                        Ok(()) => {
                            // Despite doing all these, if the new app's start
                            // address and size make it such that it will cross
                            // the bounds of flash, we return a No Memory error.
                            if new_address + (app_length - 1) > flash_end {
                                return Err(ErrorCode::NOMEM);
                            }
                            // Otherwise, we found the perfect address for our
                            // new app, let us check what kind of padding we
                            // have to write, no padding, pre padding, post
                            // padding or both pre and post padding
                            let (padding_requirement, previous_app_end_addr, next_app_start_addr) =
                                match self.compute_padding_requirement_and_neighbors(new_address) {
                                    (pr, prev_app_addr, next_app_addr) => {
                                        (pr, prev_app_addr, next_app_addr)
                                    }
                                };

                            return Ok((
                                new_address,
                                padding_requirement,
                                previous_app_end_addr,
                                next_app_start_addr,
                            ));
                        }
                        Err((new_start_addr, _e)) => {
                            // We try again from the end of the overlapping app
                            new_address = new_start_addr;
                        }
                    }
                }

                StoredInFlash::ValidApp(size) => {
                    // There is already an app at this address so we will have
                    // to skip beyond it.
                    new_address = self.next_aligned_address(new_address + size, app_length);
                }
            }
        }
        Err(ErrorCode::NOMEM)
    }
}

/// This is the callback client for the underlying physical storage driver.
impl<'a> NonvolatileStorageClient for DynamicProcessLoader<'a> {
    fn read_done(&self, _buffer: &'static mut [u8], _length: usize) {
        // We will never use this, but we need to implement this anyway.
        unimplemented!();
    }

    fn write_done(&self, buffer: &'static mut [u8], length: usize) {
        match self.state.get() {
            State::AppWrite => {
                self.state.set(State::AppWrite);
                // Switch on which user generated this callback and trigger
                // client callback.
                self.client.map(|client| {
                    client.write_app_data_done(buffer, length);
                });
            }
            State::PaddingWrite => {
                // Replace the buffer after the padding is written.
                self.reset_process_loading_metadata();
                self.buffer.replace(buffer);
            }
            State::Fail => {
                // If we failed at any of writing, we want to set the state to
                // PaddingWrite so that the callback after writing the padding
                // app will get triggererd.
                self.buffer.replace(buffer);
                if let Some(metadata) = self.process_load_metadata.get() {
                    let _ = self
                        .write_padding_app(metadata.new_app_length, metadata.new_app_start_addr);
                }
                // Clear all metadata specific to this load.
                self.reset_process_loading_metadata();
            }
            State::Setup => {
                // We have finished writing the post app padding.
                self.buffer.replace(buffer);
                self.state.set(State::AppWrite);
                // Let the client know we are done setting up.
                self.client.map(|client| {
                    client.setup_done();
                });
            }
            State::Load => {
                // We finished writing pre-padding and we need to Load the app.
                self.buffer.replace(buffer);
            }
            State::Idle => {
                self.buffer.replace(buffer);
            }
        }
    }
}

/// Callback client for the async process loader
impl<'a> ProcessLoadingAsyncClient for DynamicProcessLoader<'a> {
    fn process_loaded(&self, result: Result<(), ProcessLoadError>) {
        match result {
            Ok(()) => {
                self.state.set(State::Idle);
                self.reset_process_loading_metadata();
                self.client.map(|client| {
                    client.load_done();
                });
            }
            Err(_e) => {
                self.state.set(State::Idle);
                self.reset_process_loading_metadata();
                if config::CONFIG.debug_load_processes {
                    debug!("Load Failed.");
                }
            }
        }
    }

    fn process_loading_finished(&self) {
        if config::CONFIG.debug_load_processes {
            debug!("Processes Loaded:");
            self.procs.map(|procs| {
                for (i, proc) in procs.iter().enumerate() {
                    proc.map(|p| {
                        debug!("[{}] {}", i, p.get_process_name());
                        debug!("    ShortId: {}", p.short_app_id());
                    });
                }
            });
        }
    }
}

/// Interface exposed to the app_loader capsule
impl<'a> DynamicProcessLoading for DynamicProcessLoader<'a> {
    fn set_client(&self, client: &'static dyn DynamicProcessLoadingClient) {
        self.client.set(client);
    }

    fn setup(&self, app_length: usize) -> Result<(usize, bool), ErrorCode> {
        //TODO(?): Check if it is a newer version of an existing app. We can
        // potentially flash the new app and load it before erasing the old one.
        // What happens to the process though? Need to delete it from the
        // process array and load it back in?

        let flash_start = self.flash.get().as_ptr() as usize;
        let flash = self.flash.get();
        self.process_load_metadata
            .set(ProcessLoadMetadata::default());
        let setup_done: bool;

        if self.state.get() == State::Idle {
            self.state.set(State::Setup);
            match self.find_next_available_address(flash, app_length) {
                Ok((
                    new_app_start_address,
                    padding_requirement,
                    previous_app_end_addr,
                    next_app_start_addr,
                )) => {
                    let offset = new_app_start_address - flash_start;
                    let new_process_flash = self
                        .flash
                        .get()
                        .get(offset..offset + app_length)
                        .ok_or(ErrorCode::FAIL)?;

                    self.new_process_flash.set(new_process_flash);

                    if let Some(mut metadata) = self.process_load_metadata.get() {
                        metadata.new_app_start_addr = new_app_start_address;
                        metadata.new_app_length = app_length;
                        metadata.previous_app_end_addr = previous_app_end_addr;
                        metadata.next_app_start_addr = next_app_start_addr;
                        metadata.padding_requirement = padding_requirement;
                        self.process_load_metadata.set(metadata);
                    }

                    match padding_requirement {
                        // If we decided we need to write a padding app after
                        // the new app, we go ahead and do it.
                        PaddingRequirement::PostPad | PaddingRequirement::PreAndPostPad => {
                            // Calculating the distance between our app and
                            // either the next app.
                            let new_app_end_address = new_app_start_address + app_length;
                            let post_pad_length = next_app_start_addr - new_app_end_address;
                            setup_done = false;

                            let padding_result =
                                self.write_padding_app(post_pad_length, new_app_end_address);
                            let _ = match padding_result {
                                Ok(()) => Ok(()),
                                Err(e) => {
                                    // This means we were unable to write the
                                    // padding app.
                                    self.reset_process_loading_metadata();
                                    Err(e)
                                }
                            };
                        }
                        // Otherwise we let the client know we are done with the
                        // setup, and we are ready to write the app to flash.
                        PaddingRequirement::None | PaddingRequirement::PrePad => {
                            self.state.set(State::AppWrite);
                            setup_done = true;
                        }
                    };
                    Ok((app_length, setup_done))
                }
                Err(_err) => {
                    // Reset the state to None because we did not find any
                    // available address for this app.
                    self.reset_process_loading_metadata();
                    Err(ErrorCode::FAIL)
                }
            }
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    fn write_app_data(
        &self,
        buffer: SubSliceMut<'static, u8>,
        offset: usize,
    ) -> Result<(), ErrorCode> {
        match self.state.get() {
            State::AppWrite => {
                let res = self.write(buffer, offset);
                match res {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        // If we fail here, let us erase the app we just wrote.
                        self.state.set(State::Fail);
                        Err(e)
                    }
                }
            }
            _ => {
                // We should never enter write for the rest of the conditions,
                // so return a Busy error.
                Err(ErrorCode::BUSY)
            }
        }
    }

    fn load(&self) -> Result<(), ErrorCode> {
        // We have finished writing the last user data segment, next step is to
        // load the process.
        self.state.set(State::Load);

        if let Some(metadata) = self.process_load_metadata.get() {
            match metadata.padding_requirement {
                // If we decided we need to write a padding app before the new
                // app, we go ahead and do it.
                PaddingRequirement::PrePad | PaddingRequirement::PreAndPostPad => {
                    // Calculate the distance between our app and the previous
                    // app.
                    let previous_app_end_addr = metadata.previous_app_end_addr;
                    let pre_pad_length = metadata.new_app_start_addr - previous_app_end_addr;
                    let padding_result =
                        self.write_padding_app(pre_pad_length, previous_app_end_addr);
                    match padding_result {
                        Ok(()) => {
                            if config::CONFIG.debug_load_processes {
                                debug!("Successfully writing prepadding app");
                            }
                        }
                        Err(_e) => {
                            // This means we were unable to write the padding
                            // app.
                            self.reset_process_loading_metadata();
                        }
                    };
                }
                // We should never reach here if we are not writing a prepad
                // app.
                PaddingRequirement::None | PaddingRequirement::PostPad => {
                    if config::CONFIG.debug_load_processes {
                        debug!("No PrePad app to write.");
                    }
                }
            };
            // we've written a prepad header if required, so now it is time to
            // load the app into the process array.
            let process_flash = self.new_process_flash.take().ok_or(ErrorCode::FAIL)?;

            // Get the first eight bytes of flash to check if there is another
            // app.
            let test_header_slice = match process_flash.get(0..8) {
                Some(s) => s,
                None => {
                    // There is no header here. If we fail here, let us erase
                    // the app we just wrote.
                    let _ = self
                        .write_padding_app(metadata.new_app_length, metadata.new_app_start_addr);
                    // Clear all metadata specific to this load.
                    self.reset_process_loading_metadata();
                    return Err(ErrorCode::FAIL);
                }
            };

            // Pass the first eight bytes of the tbfheader to parse out the
            // length of the tbf header and app. We then use those values to see
            // if we have enough flash remaining to parse the remainder of the
            // header.
            let (_version, _header_length, entry_length) =
                match tock_tbf::parse::parse_tbf_header_lengths(
                    test_header_slice.try_into().or(Err(ErrorCode::FAIL))?,
                ) {
                    Ok((v, hl, el)) => (v, hl, el),
                    Err(tock_tbf::types::InitialTbfParseError::InvalidHeader(_entry_length)) => {
                        // Invalid header, return Fail error. If we fail here,
                        // let us erase the app we just wrote.
                        self.state.set(State::PaddingWrite);
                        let _ = self.write_padding_app(
                            metadata.new_app_length,
                            metadata.new_app_start_addr,
                        );
                        // Clear all metadata specific to this load.
                        self.reset_process_loading_metadata();
                        return Err(ErrorCode::FAIL);
                    }
                    Err(tock_tbf::types::InitialTbfParseError::UnableToParse) => {
                        // We are unable to parse the header, which is bad
                        // because this is the app we just flashed if we fail
                        // here, let us erase the app we just wrote.
                        self.state.set(State::PaddingWrite);
                        let _ = self.write_padding_app(
                            metadata.new_app_length,
                            metadata.new_app_start_addr,
                        );
                        // Clear all metadata specific to this load.
                        self.reset_process_loading_metadata();
                        return Err(ErrorCode::FAIL);
                    }
                };

            // Now we can get a slice which only encompasses the length of flash
            // described by this tbf header.  We will either parse this as an
            // actual app, or skip over this region.
            let entry_flash = process_flash
                .get(0..entry_length as usize)
                .ok_or(ErrorCode::FAIL)?;
            self.loader_driver.load_new_applications(entry_flash);
            Ok(())
        } else {
            Err(ErrorCode::BUSY)
        }
    }
}
