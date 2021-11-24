use core::cell::Cell;
use core::convert::TryFrom;

use kernel::errorcode::into_statuscode;
use kernel::grant::Grant;
use kernel::{debug, hil};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::OptionalCell;
use kernel::{ErrorCode, ProcessId};
use kernel::hil::wifi::{Scanner, ScannerClient};

use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::WiFi as usize;

#[derive(Default)]
pub struct App {
    subscribed: bool,
}

pub struct WiFiChip<'a> {
    driver: &'a dyn hil::wifi::Scanner<'a>,
    apps: Grant<App, 1>,
    current_process: OptionalCell<ProcessId>,
}

impl <'a> WiFiChip <'a> {
    pub fn new(
        driver: &'a dyn hil::wifi::Scanner<'a>,
        grant: Grant<App, 1>,
    ) -> WiFiChip<'a> {
        WiFiChip {
            driver: driver,
            apps: grant,
            current_process: OptionalCell::empty(),
        }
    }

    // fn enqueue_command(&self, process_id: ProcessId) -> CommandReturn {
    //     self.apps
    //         .enter(process_id, |app, _| {
    //             if !self.busy.get() {
    //                 app.subscribed = true;
    //                 self.busy.set(true);
    //                 let rcode = self.driver.start_scan_networks();
    //                 let eres = ErrorCode::try_from(rcode);
    //                 match eres {
    //                     Ok(ecode) => CommandReturn::failure(ecode),
    //                     _ => CommandReturn::success(),
    //                 }
    //             } else {
    //                 CommandReturn::failure(ErrorCode::BUSY)
    //             }
    //         })
    //         .unwrap_or_else(|err| CommandReturn::failure(err.into()))
    // }
}

use kernel::hil::wifi::Network;
impl hil::wifi::ScannerClient for WiFiChip <'_> {
    fn  scan_done<'a>(&self, status: Result<&'a [Network], ErrorCode>){
        // for cntr in self.apps.iter() {
        //     cntr.enter(|app, upcalls| {
        //         if app.subscribed {
        //             app.subscribed = false;
                    
        //             upcalls.schedule_upcall(0, (into_statuscode(status), len, 0)).ok();
        //         }
        //     });
        // }

        if let Ok(networks) = status {
            for network in networks {
                debug!("Network: {:?}", core::str::from_utf8(&network.ssid.value[0..network.ssid.len as usize]))
            }
        }
        
    }
   
}

impl SyscallDriver for WiFiChip<'_> {
    fn command( &self,
        command_num: usize,
        data1: usize,
        data2: usize,
        process_id: ProcessId,
    ) -> CommandReturn  {
        if command_num == 0 {
            return CommandReturn::success();
        }

        if self.current_process.is_none() {
            match command_num {
                1 => {
                    if let Err(err) = self.driver.scan() {
                        CommandReturn::failure(ErrorCode::FAIL)
                    } else {
                        CommandReturn::success()
                    }
                }
                _ => CommandReturn::failure(ErrorCode::NOSUPPORT),

            }
        }
        else
        {
            CommandReturn::failure(ErrorCode::BUSY)
        }

    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }

}
