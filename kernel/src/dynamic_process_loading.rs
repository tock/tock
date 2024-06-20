// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Dynamic Process Loader for OTA application loads and updates
//!
//! These functions facilitate dynamic application loading and process
//! creation during runtime without requiring the user to restart the
//! device.

use core::cell::Cell;

use crate::capabilities::ProcessManagementCapability;
use crate::config;
use crate::create_capability;
use crate::debug;
use crate::dynamic_process_metadata::{PaddingRequirement, ProcessLoadMetadata};
use crate::hil::nonvolatile_storage::{NonvolatileStorage, NonvolatileStorageClient};
use crate::kernel::Kernel;
use crate::platform::chip::Chip;
use crate::process::{self, Process, ShortId};
use crate::process_binary::{ProcessBinary, ProcessBinaryError};
use crate::process_loading::ProcessLoadError;
use crate::process_policies::ProcessFaultPolicy;
use crate::process_standard::ProcessStandard;
use crate::utilities::cells::{MapCell, OptionalCell, TakeCell};
use crate::utilities::leasable_buffer::SubSliceMut;
use crate::ErrorCode;

const MAX_PROCS: usize = 10; //need this to store the start addresses of processes to write padding
pub const BUF_LEN: usize = 512;
const TBF_HEADER_LENGTH: usize = 16;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum State {
    Idle,
    Setup,
    AppWrite,
    Load,
    PaddingWrite,
    Fail,
}

/// This interface supports loading processes at runtime.
pub trait DynamicProcessLoading {
    /// Call to request loading a new process.
    ///
    /// This informs the kernel we want to load a process and the size of the entire process binary.
    /// The kernel will try to find a suitable location in flash to store said process.
    ///
    /// Return value:
    /// - `Ok((length, wait_for_setup))`: If there is a place to load the
    ///   process, the function will return `Ok()` with the size of the region
    ///   to store the process, and whether the process loader is waiting to
    ///   set up. This usually happens when we have to write a post pad app.
    ///   The client app is unable to write new app data until the process loader
    ///   finishes writing the padding app. So if tihs flag is set, then the
    ///   client app has to wait until the setup_done subscribe callback is received.
    /// - `Err(ErrorCode)`: If there is nowhere to store the process a suitable
    ///   `ErrorCode` will be returned.
    fn setup(&self, app_length: usize) -> Result<(usize, bool), ErrorCode>;

    /// Instruct the kernel to write data to the flash
    /// This is used to write both userland apps and padding apps
    fn write_app_data(
        &self,
        buffer: SubSliceMut<'static, u8>,
        offset: usize,
    ) -> Result<(), ErrorCode>;

    /// Instruct the kernel to write data to the flash.
    ///
    /// `offset` is where to start writing within the region allocated
    /// for the new process binary from the `setup()` call.
    ///
    /// The caller must write the first 8 bytes of the process with valid header data. Writes
    /// must either be after the first 8 bytes or include the entire first
    /// 8 bytes.
    ///
    /// Returns an error if the write is outside of the permitted region or is
    /// writing an invalid header.
    fn load(&self) -> Result<(), ErrorCode>;

    /// Sets a client for the DynamicProcessLoading Object
    ///
    /// When the client operation is done, it calls the
    /// write_app_data_done(&self, buffer: &'static mut [u8], length: usize) function
    fn set_client(&self, client: &'static dyn DynamicProcessLoadingClient);
}

/// The callback for set_client(&self, Client).
/// The client capsule should implement this trait to handle the callback logic
pub trait DynamicProcessLoadingClient {
    fn write_app_data_done(&self, buffer: &'static mut [u8], length: usize);
    fn setup_done(&self);
}

pub struct DynamicProcessLoader<'a, C: 'static + Chip> {
    kernel: &'static Kernel,
    chip: &'static C,
    fault_policy: &'static dyn ProcessFaultPolicy,
    procs: MapCell<&'static mut [Option<&'static dyn process::Process>]>,
    flash: Cell<&'static [u8]>,
    app_memory: OptionalCell<&'static mut [u8]>,
    new_process_flash: OptionalCell<&'static [u8]>,
    flash_driver: &'a dyn NonvolatileStorage<'a>,
    buffer: TakeCell<'static, [u8]>,
    client: OptionalCell<&'static dyn DynamicProcessLoadingClient>,
    process_load_metadata: OptionalCell<ProcessLoadMetadata>,
    state: Cell<State>,
}

impl<'a, C: 'static + Chip> DynamicProcessLoader<'a, C> {
    pub fn new(
        processes: &'static mut [Option<&'static dyn process::Process>],
        kernel: &'static Kernel,
        chip: &'static C,
        flash: &'static [u8],
        fault_policy: &'static dyn ProcessFaultPolicy,
        driver: &'a dyn NonvolatileStorage<'a>,
        buffer: &'static mut [u8],
    ) -> Self {
        Self {
            procs: MapCell::new(processes),
            kernel,
            chip,
            flash: Cell::new(flash),
            app_memory: OptionalCell::empty(),
            fault_policy,
            new_process_flash: OptionalCell::empty(),
            flash_driver: driver,
            buffer: TakeCell::new(buffer),
            client: OptionalCell::empty(),
            process_load_metadata: OptionalCell::empty(),
            state: Cell::new(State::Idle),
        }
    }

    /// Needs to be set separately. When we instantiate the dynamic process loader instance,
    /// the board has not finished setting up the pre-existing processes. Doing so will result
    /// in the board reporting the amount of RAM the dynamic process loader has to work with.
    /// Therefore, we first let the board load all the processes and then tell us how much
    /// RAM is available for future apps separate from the instantiation.
    pub fn set_memory(&self, app_memory: &'static mut [u8]) {
        self.app_memory.set(app_memory);
    }

    /// Function to find the next available slot in the processes array
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

    /// Function to reset variables and states
    fn reset_process_loading_metadata(&self) {
        self.state.set(State::Idle);
        self.process_load_metadata.take();
    }

    /******************************************* NVM Logic **********************************************************/

    /// This function checks whether the new app will fit in the bounds dictated by the start address and length provided
    /// during the setup phase. This function then also computes where in flash the data should be written based on whether
    /// the call is coming during the app writing phase, or the padding phase.
    ///
    /// This function returns the physical address in flash where the write is supposed to happen.
    fn compute_address(&self, offset: usize, length: usize) -> Result<usize, ErrorCode> {
        // let mut state: State = State::Idle;
        let mut new_app_len: usize = 0;
        let mut new_app_addr: usize = 0;
        self.process_load_metadata.map(|metadata| {
            new_app_len = metadata.get_new_app_length();
            new_app_addr = metadata.get_new_app_addr();
        });
        let address = match self.state.get() {
            State::AppWrite => {
                // Check if there is an overflow while adding length and offset.
                // If there is no overflow, then we check if the length of the new write block
                // goes over the total size alloted to the new application.
                // We also check if the new app is trying to write beyond the bounds of the
                // flash region allocated to it.
                // On success, we compute the new address to write the app binary segment.
                // On failure, we return an Invalid error.
                match offset.checked_add(length) {
                    Some(result) => {
                        if length > new_app_len || result > new_app_len {
                            // this means the app is out of bounds
                            return Err(ErrorCode::INVAL);
                        }
                    }
                    None => {
                        return Err(ErrorCode::INVAL);
                    }
                }
                offset + new_app_addr
            }
            // If we are going to write the padding header, we already know where to write in flash,
            // so we don't have to add the start address
            State::Setup | State::Load | State::PaddingWrite => offset,
            // We aren't supposed to be able to write unless we
            // are in one of the first two write states
            _ => {
                return Err(ErrorCode::FAIL);
            }
        };
        Ok(address)
    }

    /// Compute the physical address where we should write the data and then write it.
    ///
    /// This function calls write_done() when the Nonvolatile Driver invokes the callback.
    ///
    /// Current limitation: There are two limitations with the current implementation of how
    /// tock looks for and loads apps.
    /// Assuming the flash looks like this:
    ///          ____________________________________________________
    ///         |             |    |            |          |         |
    ///         |     App1    | H? |   NewApp?  |   Pad    |   App2  |
    ///         |_____________|____|____________|__________|_________|
    ///
    /// Assuming the new app goes in between App 1 and App 2 which are existing processes, we
    /// write the padding after the new app during setup phase.
    /// Issue 1:        If there is a power cycle as we try to write the header for the
    ///                 new app, during the flash erase part of the flash write,
    ///                 we might end up with a break in the linkedlist when the device
    ///                 reboots and we never boot app 2.
    /// Potential Fix:  Reserve a section of flash to hold an index/repository of valid headers
    ///                 of current processes as a fall back mechanism in case we end up with
    ///                 corrupt headers.
    ///
    /// Issue 2:        If the header is written successfully, but there is a power cycle as the
    ///                 rest of the app binary is being written, we could end up with the situation
    ///                 where because the header is valid, we could end up with memory fragmentation.
    /// Potential Fix:  Create a processes monitoring process that is able to clean up after corrupt
    ///                 apps and defragment memory?
    fn write(&self, user_buffer: SubSliceMut<'static, u8>, offset: usize) -> Result<(), ErrorCode> {
        let length = user_buffer.len();
        let buffer = user_buffer.take(); // for us to perform tbf header validation and write with

        let physical_address = match self.compute_address(offset, length) {
            Ok(address) => address,
            Err(e) => return Err(e),
        };

        // The kernel needs to check if the app is trying to write/overwrite the header. So the app can only
        // write to the first 8 bytes if the app is writing all 8 bytes. Else, the kernel must raise an error.
        // The app is not allowed to write from say, offset 4 because we have to ensure the validity
        // of the header.

        // This means the app is trying to manipulate the space where the TBF header should go.
        // Ideally, we want the app to only write the complete set of 8 bytes which is used to determine
        // if the header is valid.
        // We don't apps to do this, so we return an error.

        if offset < 8 && offset != 0 {
            return Err(ErrorCode::INVAL);
        }

        if offset == 0 {
            // The app is not allowed to manipulate parts of the TBF header, so if it is trying
            // to write at the very beginning of the promised flash region, we require the app
            // writes the entire 8 bytes of the header. This header is then checked for validity.

            if length < 8 {
                return Err(ErrorCode::INVAL);
            }
            // Get a slice of the first 8 bytes
            static mut HEADER_SLICE: [u8; 8] = [0; 8];
            unsafe { HEADER_SLICE.copy_from_slice(&buffer[..8]) };

            // Convert it to an immutable slice
            let header_info: &[u8] = unsafe { &HEADER_SLICE };

            // check if there is valid information in the slice (non None values)
            let test_header_slice = match header_info.get(0..8) {
                Some(slice) => slice,
                None => {
                    // This means this is probably not a header, so return an error
                    return Err(ErrorCode::INVAL);
                }
            };

            // Pass the first eight bytes of the tbf header to parse out the length
            // of the header and app. We then use those values to see if the app
            // is going to be valid.
            let (_version, _header_length, entry_length) =
                match tock_tbf::parse::parse_tbf_header_lengths(
                    test_header_slice.try_into().or(Err(ErrorCode::FAIL))?,
                ) {
                    Ok((v, hl, el)) => (v, hl, el),
                    Err(tock_tbf::types::InitialTbfParseError::InvalidHeader(_entry_length)) => {
                        // If we have an invalid header, so we return an error
                        return Err(ErrorCode::INVAL);
                    }
                    Err(tock_tbf::types::InitialTbfParseError::UnableToParse) => {
                        // If we could not parse the header,then that's an issue. We return an Error
                        return Err(ErrorCode::INVAL);
                    }
                };

            // check if the length in the header is matching what the app requested during the setup phase
            // also check if the kernel version matches the version indicated in the new application
            let mut new_app_len = 0;
            self.process_load_metadata.map(|metadata| {
                new_app_len = metadata.get_new_app_length();
            });
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
        // write the header into the array
        self.buffer.map(|buffer| {
            //first two bytes are the kernel version
            buffer[0] = (crate::KERNEL_MAJOR_VERSION & 0xff) as u8;
            buffer[1] = ((crate::KERNEL_MAJOR_VERSION >> 8) & 0xff) as u8;

            // the next two bytes are the header length (fixed to 16 bytes for padding)
            buffer[2] = (TBF_HEADER_LENGTH & 0xff) as u8;
            buffer[3] = ((TBF_HEADER_LENGTH >> 8) & 0xff) as u8;

            // the next 4 bytes are the total app length including the header
            buffer[4] = (padding_app_length & 0xff) as u8;
            buffer[5] = ((padding_app_length >> 8) & 0xff) as u8;
            buffer[6] = ((padding_app_length >> 16) & 0xff) as u8;
            buffer[7] = ((padding_app_length >> 24) & 0xff) as u8;

            // we set the flags to 0
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
        let result = self.buffer.take().map_or(Err(ErrorCode::BUSY), |buffer| {
            if flash_end - offset >= TBF_HEADER_LENGTH {
                //write the header only if there are more than 16 bytes available in the flash
                let mut padding_slice = SubSliceMut::new(buffer);
                padding_slice.slice(..TBF_HEADER_LENGTH);
                let res = self.write(padding_slice, offset); //we are only writing the header, so 16 bytes is enough
                match res {
                    Ok(()) => Ok(()),
                    Err(e) => Err(e),
                }
            } else {
                Err(ErrorCode::NOMEM) // this means we do not have even 16 bytes to write the header
            }
        });
        match result {
            Ok(()) => Ok(()),
            Err(e) => Err(e),
        }
    }

    /// This function checks if there is a need to pad either before or after the new app to preserve
    /// the linked list.
    /// When do we pad?
    ///     1. When there is a process in the processes array which is located in flash after the new app
    ///        but not immediately after, we need to add padding between the new app and the existing app.
    ///     2. Due to MPU alignment, the new app may be similarly placed not immediately after an existing
    ///        process, in that case, we need to add padding between the previous app and the new app.
    ///     3. If both the above conditions are met, we add both a prepadding and a postpadding.
    ///     4. If either of these conditions are not met, we don't pad.
    fn compute_padding_requirement_and_neighbors(
        &self,
        new_app_start_address: usize,
    ) -> (PaddingRequirement, usize, usize) {
        // We have finished setting up for the new app successfully, so let us write the padding app

        let mut app_length = 0;
        let new_app_end_address = new_app_start_address + app_length; // the end address of our newly loaded application
        let mut next_app_start_addr = 0; // to store the address until which we need to write the padding app
        let mut previous_app_end_addr = 0; // to store the address from which we need to write the padding app
        let mut padding_requirement: PaddingRequirement = PaddingRequirement::None;

        let mut processes_start_addresses: [usize; MAX_PROCS] = [0; MAX_PROCS];
        let mut processes_end_addresses: [usize; MAX_PROCS] = [0; MAX_PROCS];

        self.process_load_metadata.map(|metadata| {
            app_length = metadata.get_new_app_length();
        });

        // get the start and end addresses in flash of existing processes
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
        // 1. If the new app is placed in between two existing processes, we compute the closest located processes
        // 2. Once we compute these values, we determine if we need to write a pre pad header, or a post pad header, or both
        // 3. If there are no apps after ours in the process array, we don't do anything

        // postpad requirement
        if let Some(next_closest_neighbor) = processes_start_addresses
            .iter()
            .filter(|&&x| x > new_app_end_address - 1)
            .min()
        {
            next_app_start_addr = *next_closest_neighbor; // we found the next closest app in flash
            if next_app_start_addr != 0 {
                padding_requirement = PaddingRequirement::PostPad;
            }
        } else {
            if config::CONFIG.debug_load_processes {
                debug!("No App Found after the new app so not adding post padding.");
            }
        }

        // prepad requirement
        if let Some(previous_closest_neighbor) = processes_end_addresses
            .iter()
            .filter(|&&x| x < new_app_start_address + 1)
            .max()
        {
            previous_app_end_addr = *previous_closest_neighbor; // we found the previous closest app in flash
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

    /******************************************* Process Load Logic **********************************************************/

    /// Check if there is a padding app at the address
    fn check_for_padding_app(&self, new_start_address: usize) -> Result<bool, ProcessBinaryError> {
        //We only need tbf header information to get the size of app which is already loaded
        let header_info = unsafe { core::slice::from_raw_parts(new_start_address as *const u8, 8) };

        let test_header_slice = match header_info.get(0..8) {
            Some(s) => s,
            None => {
                // This means we have reached the end of flash,
                // and there is nothing to test here
                return Err(ProcessBinaryError::NotEnoughFlash);
            }
        };

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
                    // If we could not parse the header, then we want to skip over
                    // this app and look for the next one.
                    return Err(ProcessBinaryError::TbfHeaderNotFound);
                }
                Err(tock_tbf::types::InitialTbfParseError::UnableToParse) => {
                    // Since Tock apps use a linked list, it is very possible the
                    // header we started to parse is intentionally invalid to signal
                    // the end of apps.
                    return Ok(false);
                }
            };

        //If a padding app is exist at the start address satisfying MPU rules, we load the new app from here!
        let header_flash = unsafe {
            core::slice::from_raw_parts(new_start_address as *const u8, header_length as usize)
        };

        let tbf_header = tock_tbf::parse::parse_tbf_header(header_flash, version)?;

        // If this isn't an app (i.e. it is padding)
        if !tbf_header.is_app() {
            return Ok(true);
        }

        Ok(false)
    }

    /// Check if the flash is empty at the address
    fn check_for_empty_flash_region(
        &self,
        new_start_address: usize,
    ) -> Result<bool, ProcessBinaryError> {
        //We only need tbf header information to get the size of app which is already loaded
        let header_info = unsafe { core::slice::from_raw_parts(new_start_address as *const u8, 8) };

        let test_header_slice = match header_info.get(0..8) {
            Some(s) => s,
            None => {
                // This means we have reached the end of flash,
                // and there is nothing to test here
                return Err(ProcessBinaryError::NotEnoughFlash);
            }
        };

        let (_version, _header_length, _entry_length) =
            match tock_tbf::parse::parse_tbf_header_lengths(
                test_header_slice
                    .try_into()
                    .or(Err(ProcessBinaryError::TbfHeaderNotFound))?,
            ) {
                Ok((v, hl, el)) => (v, hl, el),
                Err(tock_tbf::types::InitialTbfParseError::InvalidHeader(_entry_length)) => {
                    // If we could not parse the header, then we want to skip over
                    // this app and look for the next one.
                    return Err(ProcessBinaryError::TbfHeaderNotFound);
                }
                Err(tock_tbf::types::InitialTbfParseError::UnableToParse) => {
                    // Since Tock apps use a linked list, it is very possible the
                    // header we started to parse is intentionally invalid to signal
                    // the end of apps.
                    // This points to a viable start address for a new app
                    return Ok(true);
                }
            };
        Ok(false) // this means there is some data here, and we need to check if it is a remnant application
    }

    /// Check if our new app overlaps with existing apps
    fn check_overlap_region(
        &self,
        new_start_address: usize,
        app_length: usize,
    ) -> Result<(), (usize, ProcessLoadError)> {
        let new_process_count = self.find_open_process_slot().unwrap_or_default(); // find the next open process slot
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

    /// This function takes in the available flash slice and the app length specified for the new app,
    /// and returns a valid address where the new app can be flashed such that the linked list and
    /// memory alignment rules are preserved
    fn find_next_available_address(
        &self,
        flash: &'static [u8],
        app_length: usize,
    ) -> Result<(usize, PaddingRequirement, usize, usize), ErrorCode> {
        let mut new_address = flash.as_ptr() as usize; // we store the address of the new application here
        let flash_end = flash.as_ptr() as usize + flash.len() - 1;

        while new_address < flash_end {
            // iterate over the slice
            let mut is_padding_app: bool = false;
            let mut is_empty_region: bool = false;
            let mut is_remnant_region: bool = true;
            let test_address = new_address;

            let padding_result = self.check_for_padding_app(test_address); //check if there is a padding app in that space
            match padding_result {
                Ok(padding_app) => {
                    if padding_app {
                        is_padding_app = true;
                    }
                }
                Err(_e) => {
                    return Err(ErrorCode::FAIL);
                }
            }

            if !is_padding_app {
                // we check for empty region only if we do not find a padding app
                let empty_result = self.check_for_empty_flash_region(test_address); //check if the flash region is empty
                match empty_result {
                    Ok(empty_space) => {
                        if empty_space {
                            is_empty_region = true;
                        } else {
                            let new_process_count =
                                self.find_open_process_slot().unwrap_or_default(); // should never default because we have at least the dynamic app loader helper app running
                            self.procs.map(|procs| {
                                for (proc_index, value) in procs.iter().enumerate() {
                                    if proc_index < new_process_count {
                                        {
                                            // check if there is a remnant app in that space
                                            if new_address
                                                == value.unwrap().get_addresses().flash_start
                                            {
                                                // indicates there is an active process whose binary is stored here
                                                // so let us get the size of the process's binary and add that to our current
                                                // address
                                                let existing_app_end_addr =
                                                    value.unwrap().get_addresses().flash_end;

                                                // check if the new app will be aligned with its size at the end of previous app
                                                // if not, find the address where it will be aligned
                                                let result = existing_app_end_addr % app_length;
                                                new_address = if result == 0 {
                                                    existing_app_end_addr
                                                } else {
                                                    existing_app_end_addr + (app_length - result)
                                                };

                                                is_remnant_region = false;
                                            }
                                        }
                                    }
                                }
                            });
                        }
                    }
                    Err(_e) => {
                        return Err(ErrorCode::FAIL);
                    }
                }
            }

            if is_padding_app || is_empty_region || is_remnant_region {
                let address_validity_check = self.check_overlap_region(test_address, app_length);

                match address_validity_check {
                    Ok(()) => {
                        // despite doing all these, if the new app's start address and size make it such that it will
                        // cross the bounds of flash, we return a No Memory error.
                        if new_address + (app_length - 1) > flash_end {
                            return Err(ErrorCode::NOMEM);
                        }
                        // otherwise, we found the perfect address for our new app, let us
                        // check what kind of padding we have to write, no padding, pre padding,
                        // post padding or both pre and post padding
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
        }
        Err(ErrorCode::NOMEM)
    }

    //********************* Loading Process into Process Array **************************//

    fn load_processes(
        &self,
        app_flash: &'static [u8],
        app_memory: &'static mut [u8],
        _capability_management: &dyn ProcessManagementCapability,
    ) -> Result<(), ProcessLoadError> {
        let (remaining_memory, _remaining_flash) =
            self.load_processes_from_flash(app_flash, app_memory)?;

        if config::CONFIG.debug_process_credentials {
            debug!("Checking: no checking, load and run all processes");
        }
        self.procs.map(|procs| {
            for proc in procs.iter() {
                proc.map(|p| {
                    if config::CONFIG.debug_process_credentials {
                        debug!("Running {}", p.get_process_name());
                    }
                });
            }
        });
        self.app_memory.set(remaining_memory); // update our reference of remaining memory
        Ok(())
    }

    fn load_processes_from_flash(
        &self,
        app_flash: &'static [u8],
        app_memory: &'static mut [u8],
    ) -> Result<(&'static mut [u8], &'static [u8]), ProcessLoadError> {
        if config::CONFIG.debug_load_processes {
            debug!(
                "Loading processes from flash={:#010X}-{:#010X} into sram={:#010X}-{:#010X}",
                app_flash.as_ptr() as usize,
                app_flash.as_ptr() as usize + app_flash.len() - 1,
                app_memory.as_ptr() as usize,
                app_memory.as_ptr() as usize + app_memory.len() - 1
            );
        }

        let mut remaining_flash = app_flash;
        let mut remaining_memory = app_memory;
        let index = self
            .find_open_process_slot()
            .ok_or(ProcessLoadError::NoProcessSlot)?; // find the open process slot

        if config::CONFIG.debug_process_credentials {
            debug!(
                "Requested flash ={:#010X}-{:#010X}",
                remaining_flash.as_ptr() as usize,
                remaining_flash.as_ptr() as usize + remaining_flash.len() - 1
            );
        }

        let load_binary_result = self.discover_process_binary(remaining_flash);

        match load_binary_result {
            Ok((new_flash, process_binary)) => {
                remaining_flash = new_flash;

                let load_result = self.load_process(
                    process_binary,
                    remaining_memory,
                    ShortId::LocallyUnique,
                    index,
                );
                match load_result {
                    Ok((new_mem, proc)) => {
                        remaining_memory = new_mem;
                        if proc.is_some() {
                            if config::CONFIG.debug_load_processes {
                                proc.map(|p| debug!("Loaded process {}", p.get_process_name()));
                            }
                            self.procs.map(|procs| {
                                procs[index] = proc; // add the process to the processes array
                            });
                        } else {
                            if config::CONFIG.debug_load_processes {
                                debug!("No process loaded.");
                            }
                        }
                    }
                    Err((new_mem, err)) => {
                        remaining_memory = new_mem;
                        if config::CONFIG.debug_load_processes {
                            debug!("Processes load error: {:?}.", err);
                        }
                    }
                }
            }
            Err((new_flash, err)) => {
                remaining_flash = new_flash;
                match err {
                    ProcessBinaryError::NotEnoughFlash | ProcessBinaryError::TbfHeaderNotFound => {
                        if config::CONFIG.debug_load_processes {
                            debug!("No more processes to load: {:?}.", err);
                        }
                        // No more processes to load.
                    }

                    ProcessBinaryError::TbfHeaderParseFailure(_)
                    | ProcessBinaryError::IncompatibleKernelVersion { .. }
                    | ProcessBinaryError::IncorrectFlashAddress { .. }
                    | ProcessBinaryError::NotEnabledProcess
                    | ProcessBinaryError::Padding => {
                        // return an error
                        return Err(ProcessLoadError::BinaryError(
                            ProcessBinaryError::NotEnabledProcess,
                        ));
                    }
                }
            }
        }
        Ok((remaining_memory, remaining_flash))
    }

    ////////////////////////////////////////////////////////////////////////////////
    // HELPER FUNCTIONS
    ////////////////////////////////////////////////////////////////////////////////

    /// Find a process binary stored at the beginning of `flash` and create a
    /// `ProcessBinary` object if the process is viable to run on this kernel.
    fn discover_process_binary(
        &self,
        flash: &'static [u8],
    ) -> Result<(&'static [u8], ProcessBinary), (&'static [u8], ProcessBinaryError)> {
        if config::CONFIG.debug_load_processes {
            debug!(
                "Loading process binary from flash={:#010X}-{:#010X}",
                flash.as_ptr() as usize,
                flash.as_ptr() as usize + flash.len() - 1
            );
        }

        // If this fails, not enough remaining flash to check for an app.
        let test_header_slice = flash
            .get(0..8)
            .ok_or((flash, ProcessBinaryError::NotEnoughFlash))?;

        // Pass the first eight bytes to tbfheader to parse out the length of
        // the tbf header and app. We then use those values to see if we have
        // enough flash remaining to parse the remainder of the header.
        //
        // Start by converting [u8] to [u8; 8].
        let header = test_header_slice
            .try_into()
            .or(Err((flash, ProcessBinaryError::NotEnoughFlash)))?;

        let (version, header_length, app_length) =
            match tock_tbf::parse::parse_tbf_header_lengths(header) {
                Ok((v, hl, el)) => (v, hl, el),
                Err(tock_tbf::types::InitialTbfParseError::InvalidHeader(app_length)) => {
                    // If we could not parse the header, then we want to skip over
                    // this app and look for the next one.
                    (0, 0, app_length)
                }
                Err(tock_tbf::types::InitialTbfParseError::UnableToParse) => {
                    // Since Tock apps use a linked list, it is very possible the
                    // header we started to parse is intentionally invalid to signal
                    // the end of apps. This is ok and just means we have finished
                    // loading apps.
                    return Err((flash, ProcessBinaryError::TbfHeaderNotFound));
                }
            };

        // Now we can get a slice which only encompasses the length of flash
        // described by this tbf header.  We will either parse this as an actual
        // app, or skip over this region.
        let app_flash = flash
            .get(0..app_length as usize)
            .ok_or((flash, ProcessBinaryError::NotEnoughFlash))?;

        // Advance the flash slice for process discovery beyond this last entry.
        // This will be the start of where we look for a new process since Tock
        // processes are allocated back-to-back in flash.
        let remaining_flash = flash
            .get(app_flash.len()..)
            .ok_or((flash, ProcessBinaryError::NotEnoughFlash))?;

        let pb = ProcessBinary::create(app_flash, header_length as usize, version, true)
            .map_err(|e| (remaining_flash, e))?;

        Ok((remaining_flash, pb))
    }

    /// Load a process stored as a TBF process binary with `app_memory` as the RAM
    /// pool that its RAM should be allocated from. Returns `Ok` if the process
    /// object was created, `Err` with a relevant error if the process object could
    /// not be created.
    fn load_process(
        &self,
        process_binary: ProcessBinary,
        app_memory: &'static mut [u8],
        app_id: ShortId,
        index: usize,
    ) -> Result<
        (&'static mut [u8], Option<&'static dyn Process>),
        (&'static mut [u8], ProcessLoadError),
    > {
        if config::CONFIG.debug_load_processes {
            debug!(
                "Loading: process flash={:#010X}-{:#010X} ram={:#010X}-{:#010X}",
                process_binary.flash.as_ptr() as usize,
                process_binary.flash.as_ptr() as usize + process_binary.flash.len() - 1,
                app_memory.as_ptr() as usize,
                app_memory.as_ptr() as usize + app_memory.len() - 1
            );
        }

        // Need to reassign remaining_memory in every iteration so the compiler
        // knows it will not be re-borrowed.
        // If we found an actual app header, try to create a `Process`
        // object. We also need to shrink the amount of remaining memory
        // based on whatever is assigned to the new process if one is
        // created.

        // Try to create a process object from that app slice. If we don't
        // get a process and we didn't get a loading error (aka we got to
        // this point), then the app is a disabled process or just padding.
        let (process_option, unused_memory) = unsafe {
            ProcessStandard::create(
                self.kernel,
                self.chip,
                process_binary,
                app_memory,
                self.fault_policy,
                app_id,
                index,
            )
            .map_err(|(e, memory)| (memory, e))?
        };

        process_option.map(|process| {
            if config::CONFIG.debug_load_processes {
                debug!(
                    "Loading: {} [{}] flash={:#010X}-{:#010X} ram={:#010X}-{:#010X}",
                    process.get_process_name(),
                    index,
                    process.get_addresses().flash_start,
                    process.get_addresses().flash_end,
                    process.get_addresses().sram_start,
                    process.get_addresses().sram_end - 1,
                );
            }
        });
        Ok((unused_memory, process_option))
    }
}

/// This is the callback client for the underlying physical storage driver.
impl<'a, C: 'static + Chip> NonvolatileStorageClient for DynamicProcessLoader<'a, C> {
    fn read_done(&self, _buffer: &'static mut [u8], _length: usize) {
        //we will never use this, but we need to implement this anyway
        unimplemented!();
    }

    fn write_done(&self, buffer: &'static mut [u8], length: usize) {
        match self.state.get() {
            State::AppWrite => {
                self.state.set(State::AppWrite);
                // Switch on which user generated this callback and trigger client callback
                self.client.map(|client| {
                    client.write_app_data_done(buffer, length);
                });
            }
            State::PaddingWrite => {
                // replace the buffer after the padding is written
                self.reset_process_loading_metadata(); // the final reset after the padding write callback
                self.buffer.replace(buffer);
            }
            State::Fail => {
                // If we failed at any of writing, we want to set the state to PaddingWrite
                // so that the callback after writing the padding app will get triggererd
                self.buffer.replace(buffer);
                self.process_load_metadata.map(|metadata| {
                    let _ = self.write_padding_app(
                        metadata.get_new_app_length(),
                        metadata.get_new_app_addr(),
                    );
                });
                self.reset_process_loading_metadata(); // clear all metadata specific to this load
            }
            State::Setup => {
                // We have finished writing the post app padding
                self.buffer.replace(buffer);
                self.state.set(State::AppWrite);
                // let the client know we are done setting up
                self.client.map(|client| {
                    client.setup_done();
                });
            }
            State::Load => {
                // We finished writing pre-padding and we need to Load the app
                self.buffer.replace(buffer);
            }
            State::Idle => {
                self.buffer.replace(buffer);
            }
        }
    }
}

/// Interface exposed to the app_loader capsule
impl<'a, C: 'static + Chip> DynamicProcessLoading for DynamicProcessLoader<'a, C> {
    fn set_client(&self, client: &'static dyn DynamicProcessLoadingClient) {
        self.client.set(client);
    }

    fn setup(&self, app_length: usize) -> Result<(usize, bool), ErrorCode> {
        //TODO(?): Check if it is a newer version of an existing app.
        // We can potentially flash the new app and load it before erasing the old one.
        // What happens to the process though? Need to delete it from the process array and load it back in?

        let flash_start = self.flash.get().as_ptr() as usize; //start of the flash region
        let flash = self.flash.get();
        self.process_load_metadata
            .set(ProcessLoadMetadata::default());
        let padding_req: bool;

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
                        // if we decided we need to write a padding app after the new app, we go ahead and do it
                        PaddingRequirement::PostPad | PaddingRequirement::PreAndPostPad => {
                            // calculating the distance between our app and either the next app
                            let new_app_end_address = new_app_start_address + app_length;
                            let post_pad_length = next_app_start_addr - new_app_end_address;
                            padding_req = true;

                            let padding_result =
                                self.write_padding_app(post_pad_length, new_app_end_address);
                            let _ = match padding_result {
                                Ok(()) => Ok(()),
                                Err(e) => {
                                    self.reset_process_loading_metadata();
                                    Err(e) // this means we were unable to write the padding app
                                }
                            };
                        }
                        // Otherwise we let the client know we are done with the setup, and we are ready to write the app to flash
                        PaddingRequirement::None | PaddingRequirement::PrePad => {
                            self.state.set(State::AppWrite);
                            padding_req = false;
                        }
                    };
                    Ok((app_length, padding_req))
                }
                Err(_err) => {
                    self.reset_process_loading_metadata(); // reset the state to None because we did not find any available address for this app
                    Err(ErrorCode::FAIL)
                }
            }
        } else {
            Err(ErrorCode::BUSY) // this means the kernel is busy doing some other operation already.
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
                        // if we fail here, let us erase the app we just wrote
                        self.state.set(State::Fail);
                        Err(e)
                    }
                }
            }
            // We should never enter write for the rest of the conditions, so return a Busy error.
            _ => Err(ErrorCode::BUSY),
        }
    }

    fn load(&self) -> Result<(), ErrorCode> {
        self.state.set(State::Load); // We have finished writing the last user data segment, next step is to load the process
        match self.state.get() {
            State::Load => {
                self.process_load_metadata
                    .map_or(Err(ErrorCode::FAIL), |metadata| {
                        match metadata.get_padding_requirement() {
                            // if we decided we need to write a padding app before the new app, we go ahead and do it
                            PaddingRequirement::PrePad | PaddingRequirement::PreAndPostPad => {
                                // calculating the distance between our app and the previous app
                                let previous_app_end_addr = metadata.get_previous_app_end_addr();
                                let pre_pad_length =
                                    metadata.get_new_app_addr() - previous_app_end_addr;
                                let padding_result =
                                    self.write_padding_app(pre_pad_length, previous_app_end_addr);
                                match padding_result {
                                    Ok(()) => {
                                        if config::CONFIG.debug_load_processes {
                                            debug!("Successfully writing prepadding app");
                                        }
                                    }
                                    Err(_e) => {
                                        // this means we were unable to write the padding app
                                        self.reset_process_loading_metadata();
                                    }
                                };
                            }
                            // We should never reach here if we are not writing a prepad app
                            PaddingRequirement::None | PaddingRequirement::PostPad => {
                                if config::CONFIG.debug_load_processes {
                                    debug!("No PrePad app to write.");
                                }
                            }
                        };
                        // we've written a prepad header if required, so now it is time to load the app into the
                        // process array
                        let process_flash = self.new_process_flash.take().ok_or(ErrorCode::FAIL)?;
                        let remaining_memory = self.app_memory.take().ok_or(ErrorCode::FAIL)?;

                        // Get the first eight bytes of flash to check if there is another app.
                        let test_header_slice = match process_flash.get(0..8) {
                            Some(s) => s,
                            None => {
                                // There is no header here
                                // if we fail here, let us erase the app we just wrote
                                self.state.set(State::Fail);
                                // let _ = self.write_padding_app(
                                //     metadata.get_new_app_length(),
                                //     metadata.get_new_app_addr(),
                                // );
                                // self.reset_process_loading_metadata(); // clear all metadata specific to this load
                                return Err(ErrorCode::FAIL);
                            }
                        };

                        // Pass the first eight bytes of the tbfheader to parse out the length of
                        // the tbf header and app. We then use those values to see if we have
                        // enough flash remaining to parse the remainder of the header.
                        let (_version, _header_length, entry_length) =
                            match tock_tbf::parse::parse_tbf_header_lengths(
                                test_header_slice.try_into().or(Err(ErrorCode::FAIL))?,
                            ) {
                                Ok((v, hl, el)) => (v, hl, el),
                                Err(tock_tbf::types::InitialTbfParseError::InvalidHeader(
                                    _entry_length,
                                )) => {
                                    // Invalid header, return Fail error.
                                    // if we fail here, let us erase the app we just wrote
                                    self.state.set(State::PaddingWrite);
                                    let _ = self.write_padding_app(
                                        metadata.get_new_app_length(),
                                        metadata.get_new_app_addr(),
                                    );
                                    self.reset_process_loading_metadata(); // clear all metadata specific to this load
                                    return Err(ErrorCode::FAIL);
                                }
                                Err(tock_tbf::types::InitialTbfParseError::UnableToParse) => {
                                    // We are unable to parse the header, which is bad because this is the app we just flashed
                                    // if we fail here, let us erase the app we just wrote
                                    self.state.set(State::PaddingWrite);
                                    let _ = self.write_padding_app(
                                        metadata.get_new_app_length(),
                                        metadata.get_new_app_addr(),
                                    );
                                    self.reset_process_loading_metadata(); // clear all metadata specific to this load
                                    return Err(ErrorCode::FAIL);
                                }
                            };

                        // Now we can get a slice which only encompasses the length of flash
                        // described by this tbf header.  We will either parse this as an actual
                        // app, or skip over this region.
                        let entry_flash = process_flash
                            .get(0..entry_length as usize)
                            .ok_or(ErrorCode::FAIL)?;

                        let capability =
                            create_capability!(crate::capabilities::ProcessManagementCapability);

                        let res = self.load_processes(entry_flash, remaining_memory, &capability);
                        match res {
                            Ok(()) => {
                                self.reset_process_loading_metadata(); // clear all metadata specific to this load
                                Ok(())
                            } // maybe set the remaining memory here if we have to change the process_loading function anyway?
                            Err(_) => {
                                // if we fail here, let us erase the app we just wrote
                                self.state.set(State::PaddingWrite);
                                let _ = self.write_padding_app(
                                    metadata.get_new_app_length(),
                                    metadata.get_new_app_addr(),
                                );
                                self.reset_process_loading_metadata(); // clear all metadata specific to this load
                                Err(ErrorCode::FAIL)
                            }
                        }
                    })
            }
            // We should never enter Load for the rest of the conditions, so return a Busy error.
            _ => Err(ErrorCode::BUSY),
        }
    }
}
