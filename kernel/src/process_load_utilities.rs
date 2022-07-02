//! Helper functions related to Tock processes.
use crate::debug;
use crate::config;
use crate::kernel::Kernel;
use crate::platform::chip::Chip;
use crate::process::Process;
use crate::process_policies::ProcessFaultPolicy;
use crate::process_standard::ProcessStandard;
use crate::process_utilities::ProcessLoadError;
use crate::capabilities::MemoryAllocationCapability;
use crate::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use crate::syscall_driver::{CommandReturn, SyscallDriver};
use crate::process::ProcessId;
use crate::ErrorCode;

pub const DRIVER_NUM: usize = 0x10001;

//FLASH_START_ADDRESS and UNUSED_RAM_START_ADDRESS are used in order to load an application flashed from OTA_app
static mut FLASH_START_ADDRESS: usize = 0x00040000;
pub static mut UNUSED_RAM_START_ADDRESS: usize = 0x20006000;
pub static mut INDEX_OF_PROCESSES: usize = 0;

mod ro_allow {
    pub(crate) const WRITE: usize = 0;
    /// The number of allow buffers the kernel stores for this grant
    pub(crate) const COUNT: usize = 1;
}

/// Ids for read-write allow buffers
mod rw_allow {
    pub(crate) const READ: usize = 0;
    /// The number of allow buffers the kernel stores for this grant
    pub(crate) const COUNT: usize = 1;
}

/// State that is stored in each process's grant region to support process_load_utilities.
#[derive(Default)]
struct ProcLoaderData;

pub struct ProcessLoader <C:'static + Chip>{
    kernel: &'static Kernel,
    chip: &'static C, 
    fault_policy: &'static dyn ProcessFaultPolicy,
    ptr_process: *mut Option<&'static (dyn Process + 'static)>,
    process_num: usize,
    data: Grant<
        ProcLoaderData,
        UpcallCount<2>,
        AllowRoCount<{ ro_allow::COUNT }>,
        AllowRwCount<{ rw_allow::COUNT }>,
    >,
}

impl <C:'static + Chip> ProcessLoader <C> {
    pub fn init(
        kernel: &'static Kernel,
        chip: &'static C,
        fault_policy: &'static dyn ProcessFaultPolicy,
        memcapability: &dyn MemoryAllocationCapability,
        ptr_process: *mut Option<&'static (dyn Process + 'static)>,
        process_num: usize,
    ) -> ProcessLoader <C> {
        ProcessLoader {
            kernel: kernel,
            chip: chip, 
            fault_policy: fault_policy,
            ptr_process: ptr_process,
            process_num: process_num,
            data: kernel.create_grant(DRIVER_NUM, memcapability),
        }
    }

    // load_processes_advanced_air is implemented based on load_processes_advanced
    // the purpose of this function is to load an application (process) flashed from OTA_app
    fn load_processes_advanced_air(&self) -> Result<(), ProcessLoadError> { 
        // These symbols are defined in the linker script.
        extern "C" {
            /// Beginning of the ROM region containing app images.
            static mut _sapps: u8;
            /// End of the ROM region containing app images.
            static _eapps: u8;
            /// Beginning of the RAM region for app memory.
            static mut _sappmem: u8;
            /// End of the RAM region for app memory.
            static _eappmem: u8;
        }
    
        unsafe{
            debug!("flash start address {:#010X}", FLASH_START_ADDRESS);
            debug!("ram start address {:#010X}", UNUSED_RAM_START_ADDRESS);
            debug!("Index {:?}", INDEX_OF_PROCESSES);
        };
        

        let appstart = unsafe {FLASH_START_ADDRESS as *const u8};
        let appsramstart = unsafe {UNUSED_RAM_START_ADDRESS as *mut u8};

        let mut remaining_flash =  unsafe {
            core::slice::from_raw_parts(
            appstart,
            &_eapps as *const u8 as usize - appstart as usize,
        )};

        let mut remaining_memory = unsafe {
            core::slice::from_raw_parts_mut(
            appsramstart,
            &_eappmem as *const u8 as usize - appsramstart as usize,
        )};

        if unsafe { INDEX_OF_PROCESSES < self.process_num }
        {        
            // Get the first eight bytes of flash to check if there is another
            // app.
            let test_header_slice = match remaining_flash.get(0..8) {
                Some(s) => s,
                None => {
                    // Not enough flash to test for another app. This just means
                    // we are at the end of flash, and there are no more apps to
                    // load.
                    return Ok(());
                }
            };
    
            // Pass the first eight bytes to tbfheader to parse out the length of
            // the tbf header and app. We then use those values to see if we have
            // enough flash remaining to parse the remainder of the header.
            let (version, header_length, entry_length) = match tock_tbf::parse::parse_tbf_header_lengths(
                test_header_slice
                    .try_into()
                    .or(Err(ProcessLoadError::InternalError))?,
            ) {
                Ok((v, hl, el)) => (v, hl, el),
                Err(tock_tbf::types::InitialTbfParseError::InvalidHeader(entry_length)) => {
                    // If we could not parse the header, then we want to skip over
                    // this app and look for the next one.
                    (0, 0, entry_length)
                }
                Err(tock_tbf::types::InitialTbfParseError::UnableToParse) => {
                    // Since Tock apps use a linked list, it is very possible the
                    // header we started to parse is intentionally invalid to signal
                    // the end of apps. This is ok and just means we have finished
                    // loading apps.
                    return Ok(());
                }
            };

            // Now we can get a slice which only encompasses the length of flash
            // described by this tbf header.  We will either parse this as an actual
            // app, or skip over this region.
            let entry_flash = remaining_flash
                .get(0..entry_length as usize)
                .ok_or(ProcessLoadError::NotEnoughFlash)?;
        
            // Advance the flash slice for process discovery beyond this last entry.
            // This will be the start of where we look for a new process since Tock
            // processes are allocated back-to-back in flash.
            remaining_flash = remaining_flash
                .get(entry_flash.len()..)
                .ok_or(ProcessLoadError::NotEnoughFlash)?;

            // Need to reassign remaining_memory in every iteration so the compiler
            // knows it will not be re-borrowed.
            remaining_memory = if header_length > 0 {
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
                        entry_flash,
                        header_length as usize,
                        version,
                        remaining_memory,
                        self.fault_policy,
                        true,
                        INDEX_OF_PROCESSES,
                    )?
                };
                process_option.map(|process| {
                    if config::CONFIG.debug_load_processes {
                        let addresses = process.get_addresses();
                        unsafe {
                            debug!(
                            "Loaded process[{}] from flash={:#010X}-{:#010X} into sram={:#010X}-{:#010X} = {:?}",
                            INDEX_OF_PROCESSES,
                            entry_flash.as_ptr() as usize,
                            entry_flash.as_ptr() as usize + entry_flash.len() - 1,
                            addresses.sram_start,
                            addresses.sram_end - 1,
                            process.get_process_name()
                        )};
                    }
        
                    //Store the entry point of the flashed application into PROCESS global array
                    unsafe {
                        *self.ptr_process.offset(INDEX_OF_PROCESSES.try_into().unwrap()) = Some(process);
                        INDEX_OF_PROCESSES += 1;

                        let sramstart = 0x200047D4 as *mut u8;
                        let sramend= 0x200047F4 as *mut u8;
                        let test: &mut [u8] =  core::slice::from_raw_parts_mut(
                            sramstart,
                            sramend as usize - sramstart as usize,
                        );
                        debug!("2: {:?}", test);    
                    }
                });
                unused_memory
            }
            else {
                // We are just skipping over this region of flash, so we have the
                // same amount of process memory to allocate from.
                remaining_memory
            };

            //We store the start address of unused ram memory into UNUSED_RAM_START_ADDRESS (global variable)
            unsafe{
                UNUSED_RAM_START_ADDRESS = remaining_memory.as_ptr() as usize;
            }
        }

        Ok(())
    }

    // In order to match the result value of command
    fn load_processes_air(&self) -> Result<(), ErrorCode> {
        self.load_processes_advanced_air()
        .unwrap_or_else(|err| {
            debug!("Error loading processes by OTA!");
            debug!("{:?}", err);
        });

        Ok(())
    }
    
    // find_start_address_of_writable_flash_advanced is implemented based on load_processes_advanced
    // the purpose of this function is parse the start address of flash memory immediately next to the last application already loaded
    fn find_start_address_of_writable_flash_advanced(&self) -> Result<(), ProcessLoadError> {
        // These symbols are defined in the linker script.
        extern "C" {
            /// Beginning of the ROM region containing app images.
            static mut _sapps: u8;
            /// End of the ROM region containing app images.
            static _eapps: u8;
            /// Beginning of the RAM region for app memory.
            static mut _sappmem: u8;
            /// End of the RAM region for app memory.
            static _eappmem: u8;
        }

        let mut remaining_flash =  unsafe {
            core::slice::from_raw_parts(
                &_sapps as *const u8,
                &_eapps as *const u8 as usize - &_sapps as *const u8 as usize,
            )
        };

        let mut app_start_address: usize = unsafe {
            &_sapps as *const u8 as usize
        };

        let mut index = 0;

        while index < self.process_num
        {
            let test_header_slice = match remaining_flash.get(0..8) {
                Some(s) => s,
                None => {
                    // Not enough flash to test for another app. This just means
                    // we are at the end of flash, and there are no more apps to
                    // load.
                    return Ok(());
                }
            };
    
            let (_version, _header_length, entry_length) = match tock_tbf::parse::parse_tbf_header_lengths(
                test_header_slice
                    .try_into()
                    .or(Err(ProcessLoadError::InternalError))?,
            ) {
                Ok((v, hl, el)) => (v, hl, el),
                Err(tock_tbf::types::InitialTbfParseError::InvalidHeader(entry_length)) => {
                    // If we could not parse the header, then we want to skip over
                    // this app and look for the next one.
                    (0, 0, entry_length)
                }
                Err(tock_tbf::types::InitialTbfParseError::UnableToParse) => {
                    // Since Tock apps use a linked list, it is very possible the
                    // header we started to parse is intentionally invalid to signal
                    // the end of apps. This is ok and just means we have finished
                    // loading apps.
                    return Ok(());
                }
            };

            app_start_address += entry_length as usize;
            
            //FLASH_START_ADDRESS indicates the start point immediately next to the last application which is already loaded in flash memroy
            unsafe { FLASH_START_ADDRESS = app_start_address }; 
            //debug!("debug point {:?}", app_start_address);

            remaining_flash = unsafe {
                core::slice::from_raw_parts(
                    app_start_address as *const u8,
                    &_eapps as *const u8 as usize - app_start_address,
                )
            };

            index += 1;
        }

        Ok(())
    }

    // In order to match the result value of command
    fn find_start_address_of_writable_flash(&self) -> Result<(), ErrorCode> {
        self.find_start_address_of_writable_flash_advanced()
        .unwrap_or_else(|err| {
            debug!("Error finding writable flash start address!");
            debug!("{:?}", err);
        });

        Ok(())
    }

    fn cal_crc32_poxis(&self) -> u32 {
        
        let appstart = unsafe {FLASH_START_ADDRESS as *const u8};

        let header_slice =  unsafe {
            core::slice::from_raw_parts(
            appstart,
            8,
        )};
       
        let entry_length = usize::from_le_bytes([header_slice[4], header_slice[5], header_slice[6], header_slice[7]]);

        unsafe{
            debug!("crc start address {:#010X}", FLASH_START_ADDRESS);
            debug!("length {:#010X}", entry_length);
        };
        
        let data =  unsafe {
            core::slice::from_raw_parts(
            appstart,
            entry_length,
        )};

        let crc32_ref = tickv::crc32::Crc::new();
        let mut crc32_digest = crc32_ref.digest();
        crc32_digest.update(data);

        let crc32_rst = crc32_digest.finalise();

        return crc32_rst;
    }
}

impl <C:'static + Chip> SyscallDriver for ProcessLoader <C> {
    /// ### `command_num`
    ///
    /// - `0`: Driver check, always returns Ok(())
    /// - `1`: Perform loading an process flashed from OTA_app and write the entry point of the process into PROCESS global array
    /// - `2`: Perform finding the start address of flash memory immediately next to the last application already loaded
    /// - `3`: Return the start address of flash memory 
    /// - `4`: Return the number of processes
    /// - `5`: Return the result value of CRC32-POXIS
    
    fn command(
        &self,
        command_num: usize,
        _unused1: usize,
        _unused2: usize,
        _unused3: ProcessId,
    ) -> CommandReturn {
        match command_num {
            0 => CommandReturn::success(),
            
            1 =>
            /* Discover */
            {
                let res = self.load_processes_air();

                match res {
                    Ok(()) => CommandReturn::success(),
                    Err(e) => CommandReturn::failure(e),
                }
            }

            2 =>
            /* find start address of writable flash memory immediately next to the last application already loaded */
            {
                let res = self.find_start_address_of_writable_flash();
                
                match res {
                    Ok(()) => CommandReturn::success(),
                    Err(e) => CommandReturn::failure(e),
                }
            }

            3 =>
            /* return start address of the writable flash memory */
            {
                CommandReturn::success_u32(unsafe {FLASH_START_ADDRESS} as u32)
            }

            4 =>
            /* return the number of processes */
            {
                CommandReturn::success_u32(unsafe {INDEX_OF_PROCESSES} as u32)
            }

            5 =>
            /* return crc32 value of a process installed by OTA_app */
            {
                CommandReturn::success_u32(self.cal_crc32_poxis() as u32)
            }

            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), crate::process::Error> {
        self.data.enter(processid, |_, _| {})
    }
}