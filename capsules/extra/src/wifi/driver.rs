// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

use super::{Client, Device};
use crate::wifi::{len, Security, Ssid, Wpa3Passphrase, WpaPassphrase};
use kernel::errorcode::into_statuscode;
use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::processbuffer::{ReadableProcessBuffer, WriteableProcessBuffer};
use kernel::syscall::{self, CommandReturn, SyscallDriver};
use kernel::{process, utilities::cells::OptionalCell, ErrorCode, ProcessId};

/// Ids for read-only allow buffers
mod ro_allow {
    pub const SSID: usize = 0;
    pub const PASS: usize = 1;

    /// The number of RO allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 2;
}

/// Ids for read-write allow buffers
mod rw_allow {
    pub const MAC: usize = 0;
    pub const SCAN_SSID: usize = 1;

    /// The length for the MAC address buffer
    pub const MAC_LEN: usize = 6;
    /// The number of RW allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 2;
}

/// Ids for upcalls
mod upcall {
    pub const INIT: usize = 0;
    pub const JOIN: usize = 1;
    pub const LEAVE: usize = 2;
    pub const AP: usize = 3;
    pub const STA: usize = 4;
    pub const SCAN: usize = 5;
    pub const STOP_SCAN: usize = 6;

    pub const SCAN_RES: usize = 7;
    pub const SCAN_DONE: usize = 8;

    /// The number of upcalls the kernel stores for this grant
    pub const COUNT: u8 = 9;
}

/// Wifi security options encodings
mod security {
    pub const OPEN: usize = 0;
    pub const WPA: usize = 1;
    pub const WPA2: usize = 2;
    pub const WPA2_WPA3: usize = 3;
    pub const WPA3: usize = 4;
}

#[repr(usize)]
#[derive(Clone, Copy, Debug)]
pub enum Command {
    Init = upcall::INIT,
    Join = upcall::JOIN,
    Leave = upcall::LEAVE,
    Ap = upcall::AP,
    Sta = upcall::STA,
    Scan = upcall::SCAN,
    StopScan = upcall::STOP_SCAN,
}

impl Command {
    #[inline]
    fn to_upcall(self) -> usize {
        self as _
    }
}

#[derive(Default)]
pub struct App;

pub struct WifiDriver<'a, D: Device<'a>> {
    device: &'a D,
    process_id: OptionalCell<ProcessId>,
    command: OptionalCell<Command>,
    grants: Grant<
        App,
        UpcallCount<{ upcall::COUNT }>,
        AllowRoCount<{ ro_allow::COUNT }>,
        AllowRwCount<{ rw_allow::COUNT }>,
    >,
}

impl<'a, D: Device<'a>> WifiDriver<'a, D> {
    pub fn new(
        device: &'a D,
        grants: Grant<
            App,
            UpcallCount<{ upcall::COUNT }>,
            AllowRoCount<{ ro_allow::COUNT }>,
            AllowRwCount<{ rw_allow::COUNT }>,
        >,
    ) -> Self {
        WifiDriver {
            device,
            process_id: OptionalCell::empty(),
            command: OptionalCell::empty(),
            grants,
        }
    }

    fn credentials(
        &self,
        process_id: ProcessId,
        security: usize,
    ) -> Result<(Ssid, Option<Security>), ErrorCode> {
        self.grants.enter(process_id, |_, kernel_data| {
            let ssid = kernel_data
                .get_readonly_processbuffer(ro_allow::SSID)
                .and_then(|buf| {
                    buf.enter(|buf| {
                        if buf.len() > len::SSID {
                            return Err(ErrorCode::SIZE);
                        }
                        let mut ssid = Ssid {
                            len: buf.len() as _,
                            ..Default::default()
                        };
                        buf.copy_to_slice(&mut ssid.buf[..buf.len()]);
                        Ok(ssid)
                    })
                })
                .map_err(ErrorCode::from)??;
            let security = match security {
                security::OPEN => None,
                wpa @ security::WPA..security::WPA3 => Some(
                    kernel_data
                        .get_readonly_processbuffer(ro_allow::PASS)
                        .and_then(|buf| {
                            buf.enter(|buf| {
                                // WPA passphrase
                                if wpa <= security::WPA2 && buf.len() < len::WPA_PASSPHRASE {
                                    let mut passphrase = WpaPassphrase {
                                        len: buf.len() as _,
                                        ..Default::default()
                                    };
                                    passphrase.len = buf.len() as _;
                                    buf.copy_to_slice(&mut passphrase.buf[..buf.len()]);
                                    if wpa == security::WPA {
                                        return Ok(Security::Wpa(passphrase));
                                    } else {
                                        return Ok(Security::Wpa2(passphrase));
                                    }
                                }
                                // WPA3 passphrase
                                if wpa >= security::WPA2_WPA3 && buf.len() < len::WPA3_PASSPHRASE {
                                    let mut passphrase = Wpa3Passphrase {
                                        len: buf.len() as _,
                                        ..Default::default()
                                    };
                                    buf.copy_to_slice(&mut passphrase.buf[..buf.len()]);
                                    if wpa == security::WPA2_WPA3 {
                                        return Ok(Security::Wpa2Wpa3(passphrase));
                                    } else {
                                        return Ok(Security::Wpa3(passphrase));
                                    }
                                }
                                Err(ErrorCode::INVAL)
                            })
                        })??,
                ),
                _ => Err(ErrorCode::INVAL)?,
            };
            Ok((ssid, security))
        })?
    }
}

impl<'a, D: Device<'a>> SyscallDriver for WifiDriver<'a, D> {
    fn command(
        &self,
        command_num: usize,
        security: usize,
        channel: usize,
        process_id: ProcessId,
    ) -> syscall::CommandReturn {
        match command_num {
            0 => CommandReturn::success(),
            // Initialize the device
            1 => {
                let rval = self.device.init();
                if rval.is_ok() {
                    self.command.set(Command::Init);
                    self.process_id.set(process_id);
                }
                rval.into()
            }
            // Get the MAC address of the device.
            // Should be available if the initialisation is successful
            2 => self.device.mac().map_or_else(
                |err| Err(err).into(),
                |mac| {
                    self.grants
                        .enter(process_id, |_, kernel_data| {
                            kernel_data
                                .get_readwrite_processbuffer(rw_allow::MAC)
                                .and_then(|buf| {
                                    buf.mut_enter(|buf| {
                                        let len = usize::min(rw_allow::MAC_LEN, buf.len());
                                        buf[..len].copy_from_slice(&mac[..len]);
                                    })
                                })
                                .map_err(ErrorCode::from)
                        })
                        .unwrap_or_else(|err| Err(err.into()))
                        .into()
                },
            ),
            // Configure the device as an access point
            3 => {
                if channel > u8::MAX as _ {
                    return Err(ErrorCode::INVAL).into();
                }
                let rval = self
                    .credentials(process_id, security)
                    .and_then(|(ssid, security)| {
                        self.device.access_point(ssid, security, channel as _)
                    });
                if rval.is_ok() {
                    self.command.set(Command::Ap);
                    self.process_id.set(process_id);
                }
                rval.into()
            }
            // Configure the device as station
            4 => {
                let rval = self.device.station();
                if rval.is_ok() {
                    self.command.set(Command::Sta);
                    self.process_id.set(process_id);
                }
                rval.into()
            }
            // Join a network
            5 => {
                let rval = self
                    .credentials(process_id, security)
                    .and_then(|(ssid, security)| self.device.join(ssid, security));
                if rval.is_ok() {
                    self.command.set(Command::Join);
                    self.process_id.set(process_id);
                }
                rval.into()
            }
            // Leave the current connected network
            6 => {
                let rval = self.device.leave();
                if rval.is_ok() {
                    self.command.set(Command::Leave);
                    self.process_id.set(process_id);
                }
                rval.into()
            }
            // Start scanning for networks
            7 => {
                let rval = self.device.scan();
                if rval.is_ok() {
                    self.command.set(Command::Scan);
                    self.process_id.set(process_id);
                }
                rval.into()
            }
            // Stop scanning
            8 => {
                let rval = self.device.stop_scan();
                if rval.is_ok() {
                    self.command.set(Command::StopScan);
                    self.process_id.set(process_id);
                }
                rval.into()
            }
            _ => syscall::CommandReturn::failure(ErrorCode::INVAL),
        }
    }

    fn allocate_grant(&self, process_id: ProcessId) -> Result<(), process::Error> {
        self.grants.enter(process_id, |_, _| {})
    }
}

impl<'a, D: Device<'a>> Client for WifiDriver<'a, D> {
    fn command_done(&self, rval: Result<(), ErrorCode>) {
        Option::zip(self.process_id.get(), self.command.take()).map(|(process_id, command)| {
            let _ = self.grants.enter(process_id, |_, kernel_data| {
                let _ =
                    kernel_data.schedule_upcall(command.to_upcall(), (into_statuscode(rval), 0, 0));
                if let Command::Scan = command {
                    self.process_id.set(process_id);
                }
            });
        });
    }

    // TODO: Maybe set the same callback for `scan_done` and `scanned_network` and just set the len
    // to 0 when scanning is done.
    fn scan_done(&self) {
        self.process_id.map(|process_id| {
            self.grants.enter(process_id, |_, kernel_data| {
                let _ = kernel_data.schedule_upcall(upcall::SCAN_DONE, (0, 0, 0));
            })
        });
    }

    fn scanned_network(&self, ssid: Ssid) {
        self.process_id.get().map(|process_id| {
            self.grants
                .enter(process_id, |_, kernel_data| {
                    let _ = kernel_data
                        .get_readwrite_processbuffer(rw_allow::SCAN_SSID)
                        .and_then(|buf| {
                            buf.mut_enter(|buf| {
                                let len = usize::min(ssid.len as _, buf.len());
                                buf[..len].copy_from_slice(&ssid.buf[..len]);
                            })
                        });
                    let _ = kernel_data.schedule_upcall(upcall::SCAN_RES, (ssid.len as _, 0, 0));
                })
                .unwrap();
        });
    }
}
