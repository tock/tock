use core::mem;

use kernel::errorcode::into_statuscode;
use kernel::grant::Grant;
use kernel::hil;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::OptionalCell;
use kernel::{ErrorCode, ProcessId};

use crate::driver;
use kernel::processbuffer::{ReadWriteProcessBuffer, WriteableProcessBuffer};
pub const DRIVER_NUM: usize = driver::NUM::WiFiNina as usize;

#[derive(Default)]
pub struct App {
    wifi_networks_buffer: ReadWriteProcessBuffer,
}

pub struct WiFiChip<'a> {
    driver: &'a dyn hil::wifinina::Scanner<'a>,
    apps: Grant<App, 1>,
    current_process: OptionalCell<ProcessId>,
}

impl<'a> WiFiChip<'a> {
    pub fn new(driver: &'a dyn hil::wifinina::Scanner<'a>, grant: Grant<App, 1>) -> WiFiChip<'a> {
        WiFiChip {
            driver: driver,
            apps: grant,
            current_process: OptionalCell::empty(),
        }
    }
}

use kernel::hil::wifinina::Network;
impl hil::wifinina::ScannerClient for WiFiChip<'_> {
    fn scan_done<'a>(&self, status: Result<&'a [Network], ErrorCode>) {
        self.current_process.map(|process_id| {
            match status {
                Ok(networks) => {
                    let _ = self.apps.enter(*process_id, |app, upcalls| {
                        let _ = app.wifi_networks_buffer.mut_enter(|buffer| {
                            let mut position = 0;
                            let mut len = 0;
                            for network in networks {
                                if position + 35 > buffer.len() {
                                    break;
                                }

                                for (s, d) in
                                    network.ssid.value.iter().zip(buffer.iter().skip(position))
                                {
                                    d.set(*s);
                                }
                                buffer[position + network.ssid.len as usize].set(0);
                                buffer[position + network.ssid.len as usize + 1].set(0);
                                buffer[position + network.ssid.len as usize + 2].set(0);
                                position = position + 35;
                                len = len + 1;
                            }
                            upcalls.schedule_upcall(0, (0, len, networks.len())).ok()
                        });
                    });
                }
                Err(error) => {
                    self.apps
                        .enter(*process_id, |_app, upcalls| {
                            upcalls
                                .schedule_upcall(0, (into_statuscode(Err(error)), 0, 0))
                                .ok()
                        })
                        .ok();
                }
            }
            self.current_process.clear();
        });
    }
}

impl SyscallDriver for WiFiChip<'_> {
    fn allow_readwrite(
        &self,
        appid: ProcessId,
        allow_num: usize,
        mut slice: ReadWriteProcessBuffer,
    ) -> Result<ReadWriteProcessBuffer, (ReadWriteProcessBuffer, ErrorCode)> {
        match allow_num {
            0 => {
                let res = self
                    .apps
                    .enter(appid, |app, _| {
                        mem::swap(&mut app.wifi_networks_buffer, &mut slice);
                    })
                    .map_err(ErrorCode::from);
                match res {
                    Err(e) => Err((slice, e)),
                    Ok(_) => Ok(slice),
                }
            }
            _ => Err((slice, ErrorCode::NOSUPPORT)),
        }
    }

    fn command(
        &self,
        command_num: usize,
        _data1: usize,
        _data2: usize,
        process_id: ProcessId,
    ) -> CommandReturn {
        if command_num == 0 {
            return CommandReturn::success();
        }

        if self.current_process.is_none() {
            match command_num {
                1 => {
                    if let Err(_err) = self.driver.scan() {
                        CommandReturn::failure(ErrorCode::FAIL)
                    } else {
                        self.current_process.replace(process_id);
                        CommandReturn::success()
                    }
                }
                _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
            }
        } else {
            CommandReturn::failure(ErrorCode::BUSY)
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}
