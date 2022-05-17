use core::mem;

use kernel::errorcode::into_statuscode;
use kernel::grant::Grant;
use kernel::grant::{AllowRoCount, AllowRwCount, UpcallCount};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::OptionalCell;
use kernel::{debug, hil};
use kernel::{ErrorCode, ProcessId};

use enum_primitive::cast::FromPrimitive;
use enum_primitive::enum_from_primitive;

use crate::driver;
use kernel::processbuffer::{ReadableProcessBuffer, WriteableProcessBuffer};
pub const DRIVER_NUM: usize = driver::NUM::WiFiNina as usize;

/// Ids for read-only allow buffers
mod ro_allow {
    pub const SSID: usize = 0;
    pub const PSK: usize = 1;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: usize = 2;
}

/// Ids for read-write allow buffers
mod rw_allow {
    /// Allow a buffer for the multi touch. See header for format
    // Buffer data format
    //  0                      33              34              35                 ...
    // +---------+-------------+---------------+---------------+---------------------+-----------+ ...
    // | ssid (SSID)           | rssi (u8)     | security (u8) |                     |           |        ...
    // +---------+-------------+---------------+---------------+---------------------+---------- ...
    // | Network 0                                             | Network 1

    pub const NETWORKS: usize = 0;
    pub const SSIDS: usize = 1;
    pub const PSK: usize = 2;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: usize = 1;
}

enum_from_primitive! {
    #[derive(Debug, PartialEq, Clone, Copy)]
    /// This enum defines the command syscalls that are supported by this capsule
    pub enum Cmd {
        /// Check if driver is present
        Ping = 0,
        /// Scan network for wifis
        ScanNetworks = 1,
        /// Connect to network
        NetworkConnect = 2,
        /// Get IP Address
        IpAddress = 3
    }
}

pub trait WiFi<'a>: hil::wifinina::Scanner<'a> + hil::wifinina::Station<'a> {}
impl<'a, W: hil::wifinina::Scanner<'a> + hil::wifinina::Station<'a>> WiFi<'a> for W {}
#[derive(Default)]
pub struct App {}

pub struct WiFiChip<'a> {
    driver: &'a (dyn WiFi<'a> + 'a),
    apps: Grant<
        App,
        UpcallCount<3>,
        AllowRoCount<{ ro_allow::COUNT }>,
        AllowRwCount<{ rw_allow::COUNT }>,
    >,
    current_process: OptionalCell<ProcessId>,
}

impl<'a> WiFiChip<'a> {
    pub fn new(
        driver: &'a (dyn WiFi<'a> + 'a),
        grant: Grant<
            App,
            UpcallCount<3>,
            AllowRoCount<{ ro_allow::COUNT }>,
            AllowRwCount<{ rw_allow::COUNT }>,
        >,
    ) -> WiFiChip<'a> {
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
        debug!("Scan is done");
        self.current_process.map(|process_id| {
            match status {
                Ok(networks) => {
                    let _ = self.apps.enter(*process_id, |app, kernel_data| {
                        let _ = kernel_data
                            .get_readwrite_processbuffer(rw_allow::NETWORKS)
                            .and_then(|wifi_networks_buffer| {
                                wifi_networks_buffer.mut_enter(|buffer| {
                                    let mut position = 0;
                                    let mut len = 0;
                                    for network in networks {
                                        if position + 35 > buffer.len() {
                                            break;
                                        }

                                        for (s, d) in network
                                            .ssid
                                            .value
                                            .iter()
                                            .zip(buffer.iter().skip(position))
                                        {
                                            d.set(*s);
                                        }
                                        buffer[position + network.ssid.len as usize].set(0);
                                        buffer[position + network.ssid.len as usize + 1].set(0);
                                        buffer[position + network.ssid.len as usize + 2].set(0);
                                        position = position + 35;
                                        len = len + 1;
                                    }
                                    kernel_data
                                        .schedule_upcall(1, (0, len, networks.len()))
                                        .ok()
                                })
                            })
                            .unwrap();
                    });
                }
                Err(error) => {
                    self.apps
                        .enter(*process_id, |_app, upcalls| {
                            upcalls
                                .schedule_upcall(1, (into_statuscode(Err(error)), 0, 0))
                                .ok()
                        })
                        .ok();
                }
            }
            self.current_process.clear();
        });
    }
}

use kernel::hil::wifinina::{Station, StationClient, StationStatus};
impl hil::wifinina::StationClient for WiFiChip<'_> {
    fn command_complete(&self, status: Result<StationStatus, ErrorCode>) {
        self.current_process.map(|process_id| {
            let _ = self
                .apps
                .enter(*process_id, |app, kernel_data| match status {
                    Ok(station_status) => {
                        kernel_data
                            .schedule_upcall(
                                2,
                                (
                                    match station_status {
                                        StationStatus::Off => 0,
                                        StationStatus::Connected(_) => 1,
                                        StationStatus::Connecting(_) => 2,
                                        StationStatus::Disconnected => 3,
                                        StationStatus::Disconnecting => 4,
                                    },
                                    0,
                                    0,
                                ),
                            )
                            .ok();
                    }
                    Err(e) => {
                        debug!("Error at command_complete: {:?}", e)
                    }
                });
        });
    }
}

use kernel::hil::wifinina::{Psk, Ssid};
impl SyscallDriver for WiFiChip<'_> {
    fn command(
        &self,
        command_num: usize,
        data1: usize,
        data2: usize,
        appid: ProcessId,
    ) -> CommandReturn {
        let res = self.apps.enter(appid, |app, kernel_data| {
            if let Some(cmd) = Cmd::from_usize(command_num) {
                match cmd {
                    Cmd::Ping => {
                        debug!("Wifina driver available!");
                        CommandReturn::success()
                    }
                    Cmd::ScanNetworks => {
                        if let Err(_err) = self.driver.scan() {
                            CommandReturn::failure(ErrorCode::FAIL)
                        } else {
                            self.current_process.replace(appid);
                            CommandReturn::success()
                        }
                    }
                    Cmd::NetworkConnect => {
                        let ssid_len = data1;
                        let psk_len = data2;
                        if ssid_len > 32 || psk_len > 63 {
                            CommandReturn::failure(ErrorCode::INVAL)
                        } else {
                            if let Ok(c) = kernel_data
                                .get_readonly_processbuffer(ro_allow::SSID)
                                .and_then(|ssid_buffer| {
                                    ssid_buffer.enter(|data| {
                                        let mut buff: [u8; 32] = [0; 32];
                                        for (i, c) in data[0..ssid_len].iter().enumerate() {
                                            buff[i] = c.get();
                                        }
                                        let mut ssid: Ssid = Ssid {
                                            len: ssid_len as u8,
                                            value: buff,
                                        };

                                        let mut passv2: [u8; 63] = [0; 63];
                                        for (i, c) in
                                            data[ssid_len..ssid_len + psk_len].iter().enumerate()
                                        {
                                            passv2[i] = c.get();
                                        }
                                        let mut psk: Psk = Psk {
                                            len: 13,
                                            value: passv2,
                                        };

                                        if let Err(_err) = self.driver.connect(ssid, Some(psk)) {
                                            CommandReturn::failure(ErrorCode::FAIL)
                                        } else {
                                            self.current_process.replace(appid);
                                            CommandReturn::success()
                                        }
                                    })
                                })
                            {
                                c
                            } else {
                                CommandReturn::failure(ErrorCode::FAIL)
                            }
                            // .unwrap_or_else(CommandReturn::failure(ErrorCode::FAIL))
                            // match res {
                            //     Ok(val) => val,
                            //     _ => CommandReturn::failure(ErrorCode::FAIL)
                            // }
                        }
                    }
                    _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
                }
            } else {
                CommandReturn::failure(ErrorCode::NOSUPPORT)
            }
        });

        // .map_err(ErrorCode::from)
        match res {
            Ok(v) => v,
            // Ok(Err(e)) => CommandReturn::failure(e),
            Err(e) => CommandReturn::failure(e.into()),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}
