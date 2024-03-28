// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Dynamic Process Loader for OTA application loads and updates
//!
//! These functions facilitate dynamic application loading and process
//! creation during runtime without requiring the user to restart the
//! device.

use core::cell::Cell;
use core::cmp;

use crate::capabilities::ProcessManagementCapability;
use crate::config;
use crate::create_capability;
use crate::debug;
use crate::hil::nonvolatile_storage::{NonvolatileStorage, NonvolatileStorageClient};
use crate::kernel::Kernel;
use crate::platform::chip::Chip;
use crate::process::ProcessId;
use crate::process::{self, Process, ShortId};
use crate::process_binary::{ProcessBinary, ProcessBinaryError};
use crate::process_loading::ProcessLoadError;
use crate::process_policies::ProcessFaultPolicy;
use crate::process_standard::ProcessStandard;
use crate::utilities::cells::{MapCell, OptionalCell, TakeCell};
use crate::ErrorCode;

const TBF_HEADER_LENGTH: usize = 16;
pub const BUF_LEN: usize = 512;
const MAX_PROCS: usize = 10; //need this to store the start addresses of processes to write padding

#[derive(Clone, Copy, PartialEq)]
pub enum NonvolatileCommand {
    UserspaceRead,
    UserspaceWrite,
}

#[derive(Clone, Copy)]
pub enum NonvolatileUser {
    Client,
    Kernel,
}

struct NewApp {
    // App size requested by userland ota app
    size: usize,
    // start_addr points the start address where the new app will be loaded
    start_addr: usize,
}

/// This interface supports loading processes at runtime.
pub trait DynamicProcessLoading {
    /// Call to request loading a new process.
    ///
    /// This informs the kernel we want to load a process and the size of the entire process binary.
    /// The kernel will try to find a suitable location in flash to store said process.
    ///
    /// Return value:
    /// - `Ok((start_address, length))`: If there is a place to load the
    ///   process, the function will return `Ok()` with the address to start at
    ///   and the size of the region to store the process.
    /// - `Err(ErrorCode)`: If there is nowhere to store the process a suitable
    ///   `ErrorCode` will be returned.
    fn setup(&self, app_length: usize) -> Result<usize, ErrorCode>;

    /// Instruct the kernel to write data to the flash
    ///
    ///This is used to write both userland apps and padding apps
    fn write_app_data(
        &self,
        flag: bool,
        buffer: &'static mut [u8],
        offset: usize,
        app_size: usize,
        processid: ProcessId,
    ) -> Result<(), ErrorCode>;

    /// Instruct the kernel to load and execute the process.
    ///
    /// This tells the kernel to try to actually execute the new process that
    /// was just installed in flash from the preceding `write()` call.
    fn load(&self) -> Result<(), ErrorCode>;

    /// Sets a client for the DynamicProcessLoading Object
    ///
    /// When the client operation is done, it calls the
    /// app_data_write_done(&self, buffer: &'static mut [u8], length: usize) function
    fn set_client(&self, client: &'static dyn DynamicProcessLoadingClient);
}

/// The callback for set_client(&self, Client).
/// The client capsule should implement this trait to handle the callback logic
pub trait DynamicProcessLoadingClient {
    fn app_data_write_done(&self, buffer: &'static mut [u8], length: usize);
}

pub struct DynamicProcessLoader<'a, C: 'static + Chip> {
    kernel: &'static Kernel,
    chip: &'static C,
    fault_policy: &'static dyn ProcessFaultPolicy,
    procs: MapCell<&'static mut [Option<&'static dyn process::Process>]>,
    flash: Cell<&'static [u8]>,
    app_memory: OptionalCell<&'static mut [u8]>,
    flash_start: Cell<usize>,
    flash_end: Cell<usize>,
    new_process_flash: OptionalCell<&'static [u8]>,
    driver: &'a dyn NonvolatileStorage<'a>,
    buffer: TakeCell<'static, [u8]>,
    new_app_start_addr: Cell<usize>,
    new_app_length: Cell<usize>,
    client: OptionalCell<&'static dyn DynamicProcessLoadingClient>,
    current_user: OptionalCell<NonvolatileUser>,
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
            flash_start: Cell::new(0),
            flash_end: Cell::new(0),
            fault_policy,
            new_process_flash: OptionalCell::empty(),
            driver: driver,
            buffer: TakeCell::new(buffer),
            new_app_start_addr: Cell::new(0),
            new_app_length: Cell::new(0),
            client: OptionalCell::empty(),
            current_user: OptionalCell::empty(),
        }
    }

    // Needs to be set separately, or breaks grants allocation
    pub fn flash_and_memory(
        &self,
        app_memory: &'static mut [u8],
        flash_start: usize,
        flash_end: usize,
    ) {
        self.app_memory.set(app_memory);
        self.flash_start.set(flash_start);
        self.flash_end.set(flash_end);
    }

    // Function to find the next available slot in the processes array
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

    /******************************************* NVM Stuff **********************************************************/

    // Check so see if we are doing something. If not, go ahead and do this
    // command. If so, this is queued and will be run when the pending
    // command completes.
    fn enqueue_command(
        &self,
        padding_flag: bool,
        command: NonvolatileCommand,
        user_buffer: &'static mut [u8],
        offset: usize,
        length: usize,
    ) -> Result<(), ErrorCode> {
        //perform this bounds check if it is not a padding header because it comes from the user application
        if !padding_flag {
            match command {
                NonvolatileCommand::UserspaceRead | NonvolatileCommand::UserspaceWrite => {
                    if offset >= self.new_app_start_addr.get()
                        || length > self.new_app_length.get()
                        || offset + length > self.new_app_length.get()
                    {
                        return Err(ErrorCode::INVAL);
                    }
                }
            }
        }

        // Calculate where we want to actually read from in the physical storage.
        let physical_address = if !padding_flag {
            offset + self.new_app_start_addr.get() // this offset comes from the app, so it starts from zero.
                                                   // thererfore we need to add the physical address of the new app
                                                   // for each write
        } else {
            offset // for padding, the kernel passes the address, so no need to add an offset
        };

        let active_len = cmp::min(length, user_buffer.len());

        match command {
            NonvolatileCommand::UserspaceRead => {
                self.driver.read(user_buffer, physical_address, active_len) // change this to no support error code?
            }
            NonvolatileCommand::UserspaceWrite => {
                self.driver.write(user_buffer, physical_address, active_len)
            }
        }
    }

    fn write_padding_app(
        &self,
        padding_app_length: usize,
        offset: usize,
        version: u16,
    ) -> Result<(), ErrorCode> {
        // write the header into the array
        self.buffer.map(|buffer| {
            //first two bytes are the kernel version
            buffer[0] = (version & 0xff) as u8;
            buffer[1] = ((version >> 8) & 0xff) as u8;

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

            for i in 16..BUF_LEN {
                buffer[i] = 0xff_u8; //creating the padding (probably unnecessary)
            }
        });

        self.current_user.set(NonvolatileUser::Kernel);
        let result = self.buffer.take().map_or(Err(ErrorCode::BUSY), |buffer| {
            if self.flash_end.get() - offset >= TBF_HEADER_LENGTH {
                //write the header only if there are more than 16 bytes available in the flash
                let res = self.enqueue_command(
                    true, //let the function know that this is the padding header being written
                    NonvolatileCommand::UserspaceWrite,
                    buffer,
                    offset,
                    TBF_HEADER_LENGTH,
                ); //we are only writing the header, so 16 bytes is enough
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

    /******************************************* Process Load Stuff **********************************************************/

    fn check_for_padding_app(&self, new_app: &mut NewApp) -> Result<bool, ProcessBinaryError> {
        //We only need tbf header information to get the size of app which is already loaded
        let header_info =
            unsafe { core::slice::from_raw_parts(new_app.start_addr as *const u8, 8) };

        let test_header_slice = match header_info.get(0..8) {
            Some(s) => s,
            None => {
                // Not enough flash to test for another app. This just means
                // We are at the end of flash (0x80000).
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
                    // the end of apps. This is ok and just means we have finished
                    // loading apps.
                    return Ok(false);
                }
            };

        //If a padding app is exist at the start address satisfying MPU rules, we load the new app from here!
        let header_flash = unsafe {
            core::slice::from_raw_parts(new_app.start_addr as *const u8, header_length as usize)
        };

        let tbf_header = tock_tbf::parse::parse_tbf_header(header_flash, version)?;

        // If this isn't an app (i.e. it is padding)
        if !tbf_header.is_app() {
            return Ok(true);
        }

        Ok(false)
    }

    fn check_for_empty_flash_region(
        &self,
        new_app: &mut NewApp,
    ) -> Result<(bool, usize), ProcessBinaryError> {
        //We only need tbf header information to get the size of app which is already loaded
        let header_info =
            unsafe { core::slice::from_raw_parts(new_app.start_addr as *const u8, 8) };

        let test_header_slice = match header_info.get(0..8) {
            Some(s) => s,
            None => {
                // Not enough flash to test for another app. This just means
                // We are at the end of flash (0x80000).
                return Err(ProcessBinaryError::NotEnoughFlash);
            }
        };

        let (_version, _header_length, entry_length) =
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
                    // the end of apps. This is ok and just means we have finished
                    // loading apps.
                    // This points to a viable start address for a new app such that it satisfies MPU rules
                    return Ok((true, 0));
                }
            };
        Ok((false, entry_length as usize)) // this means there is some data here, and we need to check if it is a remnant application
    }

    // check if our new app overlaps with existing apps
    fn check_overlap_region(&self, new_app: &NewApp) -> Result<(), (usize, ProcessLoadError)> {
        let new_process_count = self.find_open_process_slot().unwrap_or_default(); // find the next open process slot
        let new_process_start_address = new_app.start_addr;
        let new_process_end_address = new_app.start_addr + new_app.size - 1;

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

    fn find_next_available_address(&self, new_app: &mut NewApp) -> Result<(), ErrorCode> {
        while new_app.start_addr < self.flash_end.get() {
            let mut is_padding_app: bool = false;
            let mut is_empty_region: bool = false;
            let mut is_remnant_region: bool = true;

            let padding_result = self.check_for_padding_app(new_app); //check if there is a padding app in that space
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
                let empty_result = self.check_for_empty_flash_region(new_app); //check if the flash region is empty
                match empty_result {
                    Ok((empty_space, size)) => {
                        if empty_space {
                            is_empty_region = true;
                        } else {
                            let new_process_count =
                                self.find_open_process_slot().unwrap_or_default(); // should never default because we have at least the OTA helper app running
                                                                                   // check if there is a remnant app in that space
                            self.procs.map(|procs| {
                                for (proc_index, value) in procs.iter().enumerate() {
                                    if proc_index < new_process_count {
                                        {
                                            if new_app.start_addr
                                                == value.unwrap().get_addresses().flash_start
                                            {
                                                // indicates there is an active process whose binary is stored here
                                                is_remnant_region = false;
                                                // Because there is an app which is also an active process, we move to the next address
                                                new_app.start_addr += cmp::max(new_app.size, size);
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
                let address_validity_check = self.check_overlap_region(new_app);

                match address_validity_check {
                    Ok(()) => {
                        // despite doing all these, if the new app's start address and size make it such that it will
                        // cross the bounds of flash, we return a No Memory error. (this is currently untested)
                        if new_app.start_addr + (new_app.size - 1) > self.flash_end.get() {
                            return Err(ErrorCode::NOMEM);
                        }
                        // otherwise, we found the perfect address for our new app
                        return Ok(());
                    }
                    Err((new_start_addr, _e)) => {
                        // We try again from the end of the overlapping app
                        new_app.start_addr = new_start_addr;
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
                        // Skip this binary and move to the next one.
                        // add a return error here
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
        // Switch on which user generated this callback.
        self.current_user.take().map(|user| {
            match user {
                NonvolatileUser::Client => {
                    // trigger the client callback
                    self.client.map(|client| {
                        client.app_data_write_done(buffer, length);
                    });
                }
                NonvolatileUser::Kernel => {
                    // replace the buffer after the padding is written
                    self.buffer.replace(buffer);
                }
            }
        });
    }
}

// Interface exposed to the app_loader capsule
impl<'a, C: 'static + Chip> DynamicProcessLoading for DynamicProcessLoader<'a, C> {
    fn set_client(&self, client: &'static dyn DynamicProcessLoadingClient) {
        self.client.set(client);
    }

    fn setup(&self, app_length: usize) -> Result<usize, ErrorCode> {
        //TODO(?): Check if it is a newer version of an existing app.
        // We can potentially flash the new app and load it before erasing the old one.
        // What happens to the process though? Need to delete it from the process array and load it back in?

        let mut new_app_data = NewApp {
            // struct to hold some information about the new app
            size: 0,
            start_addr: 0,
        };

        let flash_start = self.flash.get().as_ptr() as usize; //start of the flash region

        new_app_data.start_addr = self.flash.get().as_ptr() as usize;
        new_app_data.size = app_length;

        match self.find_next_available_address(&mut new_app_data) {
            Ok(()) => {
                let new_start_addr = new_app_data.start_addr;

                let offset = new_start_addr - flash_start;

                let new_process_flash = self
                    .flash
                    .get()
                    .get(offset..offset + app_length)
                    .ok_or(ErrorCode::FAIL)?;

                self.new_process_flash.set(new_process_flash);

                self.new_app_start_addr.set(new_app_data.start_addr);
                self.new_app_length.set(new_app_data.size);

                // reset the struct values for a new app
                new_app_data.size = 0;
                new_app_data.start_addr = 0;

                Ok(app_length)
            }
            Err(_err) => Err(ErrorCode::FAIL),
        }
    }

    fn write_app_data(
        &self,
        flag: bool,
        buffer: &'static mut [u8],
        offset: usize,
        length: usize,
        _processid: ProcessId,
    ) -> Result<(), ErrorCode> {
        if !flag {
            // this flag checks if the result is from a command that can be executed now, or if the command has been tabled for later
            self.current_user.set(NonvolatileUser::Client);
            let res = self.enqueue_command(
                false,
                NonvolatileCommand::UserspaceWrite,
                buffer,
                offset,
                length,
            );
            match res {
                Ok(()) => Ok(()),
                Err(e) => Err(e),
            }
        } else {
            // unimplemented!();   //when there are pending commands (the capsule currently returns something but we don't want it to do anything here)
            Err(ErrorCode::NOSUPPORT)
        }
    }

    fn load(&self) -> Result<(), ErrorCode> {
        let process_flash = self.new_process_flash.take().ok_or(ErrorCode::FAIL)?;
        let remaining_memory = self.app_memory.take().ok_or(ErrorCode::FAIL)?;

        // Get the first eight bytes of flash to check if there is another app.
        let test_header_slice = match process_flash.get(0..8) {
            Some(s) => s,
            None => {
                // Not enough flash to test for another app. This just means
                // we are at the end of flash, and there are no more apps to
                // load. => This case is error in loading app by ota_app, because it means that there is no valid tbf header!
                return Err(ErrorCode::FAIL);
            }
        };

        // Pass the first eight bytes to tbfheader to parse out the length of
        // the tbf header and app. We then use those values to see if we have
        // enough flash remaining to parse the remainder of the header.
        let (version, _header_length, entry_length) =
            match tock_tbf::parse::parse_tbf_header_lengths(
                test_header_slice.try_into().or(Err(ErrorCode::FAIL))?,
            ) {
                Ok((v, hl, el)) => (v, hl, el),
                Err(tock_tbf::types::InitialTbfParseError::InvalidHeader(_entry_length)) => {
                    // If we could not parse the header, then we want to skip over
                    // this app and look for the next one.
                    return Err(ErrorCode::FAIL);
                }
                Err(tock_tbf::types::InitialTbfParseError::UnableToParse) => {
                    // Since Tock apps use a linked list, it is very possible the
                    // header we started to parse is intentionally invalid to signal
                    // the end of apps. This is ok and just means we have finished
                    // loading apps. =>
                    return Err(ErrorCode::FAIL);
                }
            };

        // Now we can get a slice which only encompasses the length of flash
        // described by this tbf header.  We will either parse this as an actual
        // app, or skip over this region.
        let entry_flash = process_flash
            .get(0..entry_length as usize)
            .ok_or(ErrorCode::FAIL)?;

        let capability = create_capability!(crate::capabilities::ProcessManagementCapability);

        let res = self.load_processes(entry_flash, remaining_memory, &capability);
        let _ = match res {
            Ok(()) => Ok(()), // maybe set the remaining memory here if we have to change the process_loading function anyway?
            Err(_) => Err(ErrorCode::FAIL),
        };

        // reset the global new app start address and length values
        self.new_app_start_addr.set(0);
        self.new_app_length.set(0);

        // write padding app after our new process
        let current_process_end_addr = entry_flash.as_ptr() as usize + entry_flash.len(); // the end address of our newly loaded application
        let next_app_start_addr: usize; // to store the address until which we need to write the padding app

        let mut processes_start_addresses: [usize; MAX_PROCS] = [0; MAX_PROCS];

        self.procs.map(|procs| {
            for (procs_index, value) in procs.iter().enumerate() {
                match value {
                    Some(app) => {
                        processes_start_addresses[procs_index] = app.get_addresses().flash_start;
                    }
                    None => {
                        processes_start_addresses[procs_index] = 0;
                    }
                }
            }
        });

        // We compute the closest neighbor to our app such that:
        // 1. the neighbor app has to be placed in flash after our newly loaded app
        // 2. in the event of multiple apps in the flash after our app, we choose the one with the lowest starting address
        // if there are no apps after ours in the process array, we simply pad till the end of flash

        if let Some(closest_neighbor) = processes_start_addresses
            .iter()
            .filter(|&&x| x > current_process_end_addr)
            .min()
        {
            next_app_start_addr = *closest_neighbor; // we found the next closest app in flash
        } else {
            next_app_start_addr = self.flash_end.get(); // there are no more apps in flash, so we write padding until the end of flash
                                                        // should the padding be added only when there are other apps?
        }

        // calculating the distance between our app and either the next app, or the end of the flash
        let padding_app_length = next_app_start_addr - current_process_end_addr;

        let padding_result =
            self.write_padding_app(padding_app_length, current_process_end_addr, version);
        match padding_result {
            Ok(()) => Ok(()),
            Err(_) => Err(ErrorCode::FAIL), // this means we were unable to write the padding app
        }
    }
}
