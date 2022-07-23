//! Helper functions related to Tock processes by OTA_app. 

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

mod ro_allow {
    /// Ids for read-only allow buffers ('_' means no use)
    pub(crate) const _WRITE: usize = 0;
    /// The number of allow buffers the kernel stores for this grant
    pub(crate) const COUNT: usize = 1;
}

mod rw_allow {
    /// Ids for read-write allow buffers ('_' means no use)
    pub(crate) const _READ: usize = 0;
    /// The number of allow buffers the kernel stores for this grant
    pub(crate) const COUNT: usize = 1;
}

/// Variable that is stored in OTA_app grant region to support dynamic app load
#[derive(Default)]
struct ProcLoaderData{
    index: usize,
    dynamic_flash_start_addr: usize,
    dynamic_unsued_sram_start_addr: usize,
}

pub struct ProcessLoader <C:'static + Chip>{
    kernel: &'static Kernel,
    chip: &'static C, 
    fault_policy: &'static dyn ProcessFaultPolicy,
    ptr_process: *mut Option<&'static (dyn Process + 'static)>,
    supported_process_num: usize,
    start_app: usize,
    end_app: usize,
    end_appmem: usize,
    ptr_dynamic_unused_ram_start_addr_init: &'static usize,
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
        supported_process_num: usize,
        start_app: usize,
        end_app: usize,
        end_appmem: usize,
        ptr_dynamic_unused_ram_start_addr_init: &'static usize,
    ) -> ProcessLoader <C> {
        ProcessLoader {
            kernel: kernel,
            chip: chip, 
            fault_policy: fault_policy,
            ptr_process: ptr_process,
            supported_process_num: supported_process_num,
            start_app: start_app,
            end_app: end_app,
            end_appmem: end_appmem,
            ptr_dynamic_unused_ram_start_addr_init: ptr_dynamic_unused_ram_start_addr_init,
            data: kernel.create_grant(DRIVER_NUM, memcapability),
        }
    }

    // This function is implemented based on load_processes_advanced
    // the purpose is to load an application flashed from OTA_app into PROCESS global array
    fn load_processes_advanced_air(
        &self,
        proc_data: &mut ProcLoaderData,
    ) -> Result<(usize, Option<&'static dyn Process>), ProcessLoadError> { 

        let appstart = proc_data.dynamic_flash_start_addr as *const u8;
        let appsramstart = proc_data.dynamic_unsued_sram_start_addr as *mut u8;
        
        let mut sram_end_addresses = 0;
        let mut process_copy: Option<&'static dyn Process> = None;

        //Todo: self.eapps has to be replaced by the end address of the flahsed app? (can reduce the ram usage)
        let remaining_flash =  unsafe {
            core::slice::from_raw_parts(
            appstart,
            self.end_app - appstart as usize,
        )};

        let remaining_memory = unsafe {
            core::slice::from_raw_parts_mut(
            appsramstart,
            self.end_appmem - appsramstart as usize,
        )};

        if proc_data.index < self.supported_process_num 
        {        
            // Get the first eight bytes of flash to check if there is another
            // app.
            let test_header_slice = match remaining_flash.get(0..8) {
                Some(s) => s,
                None => {
                    // Not enough flash to test for another app. This just means
                    // we are at the end of flash, and there are no more apps to
                    // load. => This case is error in loading app by ota_app, because it means that there is no valid tbf header!
                    return Err(ProcessLoadError::InternalError);
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
                Err(tock_tbf::types::InitialTbfParseError::InvalidHeader(_entry_length)) => {
                    // If we could not parse the header, then we want to skip over
                    // this app and look for the next one. => This case is error in loading app by ota_app
                    return Err(ProcessLoadError::InternalError);
                }
                Err(tock_tbf::types::InitialTbfParseError::UnableToParse) => {
                    // Since Tock apps use a linked list, it is very possible the
                    // header we started to parse is intentionally invalid to signal
                    // the end of apps. This is ok and just means we have finished
                    // loading apps. => This case is error in loading app by ota_app
                    return Err(ProcessLoadError::InternalError);
                }
            };

            // Now we can get a slice which only encompasses the length of flash
            // described by this tbf header.  We will either parse this as an actual
            // app, or skip over this region.
            let entry_flash = remaining_flash
                .get(0..entry_length as usize)
                .ok_or(ProcessLoadError::NotEnoughFlash)?;

            // Need to reassign remaining_memory in every iteration so the compiler
            // knows it will not be re-borrowed.
            if header_length > 0 
            {
                // If we found an actual app header, try to create a `Process`
                // object. We also need to shrink the amount of remaining memory
                // based on whatever is assigned to the new process if one is
                // created.
        
                // Try to create a process object from that app slice. If we don't
                // get a process and we didn't get a loading error (aka we got to
                // this point), then the app is a disabled process or just padding.
                let (process_option, _unused_memory) = unsafe {
                    ProcessStandard::create(
                        self.kernel,
                        self.chip,
                        entry_flash,
                        header_length as usize,
                        version,
                        remaining_memory,
                        self.fault_policy,
                        true,
                        proc_data.index,
                    )?
                };
                process_option.map(|process| {
                    if config::CONFIG.debug_load_processes {
                        let addresses = process.get_addresses();
                            debug!(
                            "Loaded process[{}] from flash={:#010X}-{:#010X} into sram={:#010X}-{:#010X} = {:?}",
                            proc_data.index,
                            entry_flash.as_ptr() as usize,
                            entry_flash.as_ptr() as usize + entry_flash.len() - 1,
                            addresses.sram_start,
                            addresses.sram_end - 1,
                            process.get_process_name()
                        );
                    }
                    
                    //we return sram_end_addresses
                    let addresses = process.get_addresses();
                    sram_end_addresses = addresses.sram_end;

                    //we return process_copy
                    process_copy = Some(process);
                });
            }
            else {
                //header length 0 means invalid header
                return Err(ProcessLoadError::InternalError);
            }
        }

        Ok((sram_end_addresses, process_copy))
    }

    // In order to match the result value of command
    fn load_processes_air(
        &self,
        proc_data: &mut ProcLoaderData,
    ) -> Result<(), ErrorCode> {
        let res = self.load_processes_advanced_air(proc_data);

        //Without only alignment error, we only store the entry point of the process(app)
        match res{
            Ok((sram_end, process_copy)) => {
                //This variable will be used, when loading the another app at next load attempt by ota app
                //This is necessary to prevent the access violation of sram memory whilch are already used by kernel and other apps.
                proc_data.dynamic_unsued_sram_start_addr = sram_end;
                
                //Store the entry point of the flashed application into PROCESS global array
                //Although I used unsafe keyword, I think it's okay, becasue we pass the exact pointer of PROCESS global array
                unsafe {
                    *self.ptr_process.offset(proc_data.index.try_into().unwrap()) = process_copy;
                }

                return Ok(());
            }
            Err(_e) => {
                //debug!("Error loading processes!: {:?}", e);
                //If there is an error caused by misalignment, proc_data.dynamic_unsued_sram_start_addr will hold current unused sram start address
                return Err(ErrorCode::FAIL);
            }
        }
    }
    
    // This function is implemented based on load_processes_advanced
    // the purpose is to parse the dynamically changing start address of flash memory immediately next to the last application already loaded
    fn find_dynamic_start_address_of_writable_flash_advanced(
        &self,
        proc_data: &mut ProcLoaderData,
    ) -> Result<(), ProcessLoadError> {

        let mut app_start_address: usize = self.start_app;

        let mut remaining_flash =  unsafe {
            core::slice::from_raw_parts(
                self.start_app as *const u8,
                self.end_app - self.start_app,
            )
        };

        let mut index = 0;
        while index < self.supported_process_num
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


            // Save index data to grant region
            index += 1;
            proc_data.index = index;

            // Save dynamic_flash_start_addr to grant region
            app_start_address += entry_length as usize;
            proc_data.dynamic_flash_start_addr = app_start_address;

            // Slice data for the next while loop
            remaining_flash = unsafe {
                core::slice::from_raw_parts(
                    app_start_address as *const u8,
                    self.end_app - app_start_address,
                )
            };
        }

        Ok(())
    }

    // In order to match the result value of command
    fn find_dynamic_start_address_of_writable_flash(
        &self,
        proc_data: &mut ProcLoaderData,
    ) -> Result<(), ErrorCode> {
        self.find_dynamic_start_address_of_writable_flash_advanced(proc_data)
        .unwrap_or_else(|err| {
            debug!("Error finding writable flash start address: {:?}", err);
        });

        Ok(())
    }

    // CRC32_POSIX
    fn cal_crc32_posix(
        &self,
        proc_data: &mut ProcLoaderData,
    ) -> u32 {
        
        let appstart = proc_data.dynamic_flash_start_addr as *const u8;

        //Only parse the header information (8byte)
        let header_slice =  unsafe {
            core::slice::from_raw_parts(
            appstart,
            8,
        )};
       
        let entry_length = usize::from_le_bytes([header_slice[4], header_slice[5], header_slice[6], header_slice[7]]);
        
        let data =  unsafe {
            core::slice::from_raw_parts(
            appstart,
            entry_length,
        )};

        let mut crc32_instance = tickv::crc32::Crc32::new();
        crc32_instance.update(data);
        
        let crc32_rst = crc32_instance.finalise();

        return crc32_rst;
    }
}

impl <C:'static + Chip> SyscallDriver for ProcessLoader <C> {
    /// ### `command_num`
    ///
    /// - `0`: Driver check, always returns Ok(())
    /// - `1`: Perform loading an process flashed from OTA_app and write the entry point of the process into PROCESS global array
    /// - `2`: Perform finding the start address of flash memory immediately next to the last application already loaded
    /// - `3`: Return the dynamically changing start address after commnad 2
    /// - `4`: Initialize proc_data.dynamic_unsued_sram_start_addr with sram_end_address returned from load_processes_advanced
    ///        This initial value comes from the result value of 'kernel::process::load_processes' at main.rs (set only one time at OTA_app init stage)
    ///        This inital value is copied to internal grant variable, and this grant variable is used in 'fn load_processes_advanced_air' and updated after loading an application
    ///        We don't have to interrupt the sram region already used by kernel and other apps
    /// - `5`: Calculate CRC32-POXIS of the flashed app region and return the result value
    /// - `6`: Return the supported maximum process number by platform
    /// - `7`: Return the end address of flash memory (i.e., 0x80000 in case of this platform)
    /// - `8`: Return an index that is used to store the entry point of an app flashed into PROCESS global array
    ///        With this index, we prevent the kernel from loading 4 more than applications
    /// - `9`: Return the start address of flash memory allocated to apps (i.e., 0x40000 in case of this platform)
    
    fn command(
        &self,
        command_num: usize,
        _unused1: usize,
        _unused2: usize,
        appid: ProcessId,
    ) -> CommandReturn {
        match command_num {
            0 => CommandReturn::success(),
            
            1 =>
            /* perform load process work */
            {
                let res = self.data.enter(appid, |proc_data, _| {
                    self.load_processes_air(proc_data)
                })
                .map_err(ErrorCode::from);
        
                match res {
                    Ok(Ok(())) => CommandReturn::success(),
                    Ok(Err(e)) => CommandReturn::failure(e),
                    Err(e) => CommandReturn::failure(e),
                }
            }

            2 =>
            /* find dynamically changing start address of writable flash memory immediately next to the last application already loaded */
            {   
                let res = self.data.enter(appid, |proc_data, _| {
                    self.find_dynamic_start_address_of_writable_flash(proc_data)
                })
                .map_err(ErrorCode::from);
        
                match res {
                    Ok(Ok(())) => CommandReturn::success(),
                    Ok(Err(e)) => CommandReturn::failure(e),
                    Err(e) => CommandReturn::failure(e),
                }
            }

            3 =>
            /* Return the dynamically changing start address after commnad 2 */
            {
                self.data.enter(appid, |proc_data, _| {
                    CommandReturn::success_u32(proc_data.dynamic_flash_start_addr as u32)
                })
                .unwrap_or(CommandReturn::failure(ErrorCode::FAIL))
            }

            /* Initialize proc_data.dynamic_unsued_sram_start_addr with sram_end_address returned from load_processes_advanced */
            4 =>
            {
                let res = self.data.enter(appid, |proc_data, _| {
                    proc_data.dynamic_unsued_sram_start_addr = *self.ptr_dynamic_unused_ram_start_addr_init;
                })
                .map_err(ErrorCode::from);
        
                match res {
                    Ok(()) => CommandReturn::success(),
                    Err(e) => CommandReturn::failure(e),
                }
            }

            5 =>
            /* Calculate CRC32-POXIS of the flashed app region and return the result value */
            {
                self.data.enter(appid, |proc_data, _| {
                    let crc32 = self.cal_crc32_posix(proc_data);
                    CommandReturn::success_u32(crc32 as u32)
                })
                .unwrap_or(CommandReturn::failure(ErrorCode::FAIL))
            }

            6 =>
            /* Return the supported maximum process number by platform */
            {
                CommandReturn::success_u32(self.supported_process_num as u32)
            }

            7 =>
            /* Return the end address of flash memory (i.e., 0x80000 in case of this platform) */
            {
                CommandReturn::success_u32(self.end_app as u32)
            }

            8 =>
            /* Return index that is used to store the entry point of an app flashed */
            {
                self.data.enter(appid, |proc_data, _| {
                    CommandReturn::success_u32(proc_data.index as u32)
                })
                .unwrap_or(CommandReturn::failure(ErrorCode::FAIL))
            }

            /* Return the start address of flash memory allocated to apps (i.e., 0x40000 in case of this platform)  */
            9 =>
            {
                CommandReturn::success_u32(self.start_app as u32)
            }

            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), crate::process::Error> {
        self.data.enter(processid, |_, _| {})
    }
}