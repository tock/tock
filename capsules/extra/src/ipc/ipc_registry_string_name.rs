// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Capsule implementing the IPC Registry mechanism with string names.
//!
//! This capsule allows services to register with arbitrary 20-byte values,
//! typically strings. Capsules can discover them using those same 20-byte
//! values.
//!
//! TODO add example of how to instantiate

use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::processbuffer::ReadableProcessBuffer;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use capsules_core::driver;
pub const DRIVER_NUM: usize = driver::NUM::IpcRegistryStringName as usize;

/// Ids for read-only allow buffers
mod ro_allow {
    pub const STRING_NAME: usize = 0;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 1;
}

/// IDs for subscribed upcalls.
mod upcall {
    /// Subscribe to registration complete callback.
    pub const REGISTRATION_COMPLETE: usize = 0;
    /// Subscribe to discovery complete callback.
    pub const DISCOVERY_COMPLETE: usize = 1;
    /// Subscribe to callbacks whenever a new service registers.
    pub const NEW_REGISTRATION: usize = 2;
    /// Number of upcalls.
    pub const COUNT: u8 = 3;
}

/// Per-process metadata
#[derive(Default)]
pub struct App {
    registered_name: [u8; 20],
    get_registration_notifications: bool,
}

pub struct IpcRegistryStringName {
    apps: Grant<
        App,
        UpcallCount<{ upcall::COUNT }>,
        AllowRoCount<{ ro_allow::COUNT }>,
        AllowRwCount<0>,
    >,
}

impl IpcRegistryStringName {
    pub fn new(
        grant: Grant<
            App,
            UpcallCount<{ upcall::COUNT }>,
            AllowRoCount<{ ro_allow::COUNT }>,
            AllowRwCount<0>,
        >,
    ) -> Self {
        Self { apps: grant }
    }

    fn register(&self, processid: ProcessId) -> Result<(), ErrorCode> {
        // If registration validation is desired, that would go here before
        // saving the name

        // Save allowed name for discovery
        self.apps
            .enter(processid, |app, kerneldata| {
                kerneldata
                    .get_readonly_processbuffer(ro_allow::STRING_NAME)
                    .map_or(Err(ErrorCode::INVAL), |allow_name| {
                        allow_name
                            .enter(|buf| {
                                if buf.len() != 20 {
                                    // Error if allowed name is not exactly 20 bytes
                                    Err(ErrorCode::SIZE)
                                } else {
                                    let n = core::cmp::min(buf.len(), app.registered_name.len());
                                    buf[0..n].copy_to_slice(&mut app.registered_name[0..n]);

                                    // Schedule registration complete callback
                                    let _ = kerneldata
                                        .schedule_upcall(upcall::REGISTRATION_COMPLETE, (1, 0, 0));
                                    Ok(())
                                }
                            })
                            .unwrap_or_else(|err| err.into())
                    })
            })
            .unwrap_or_else(|err| err.into())
            .and_then(|_| {
                // TODO: Should we remove the get_registration_notification flag and commands and just notify all apps regardless? The notification would be dropped if they haven't subscribed to it.

                // Notify any other apps waiting on a registration to occur
                for cntr in self.apps.iter() {
                    if cntr.processid() != processid {
                        let otherid = cntr.processid();

                        if cntr.enter(|other_app, _| other_app.get_registration_notifications) {
                            // Notify this app
                            let _ = self.apps.enter(otherid, |_, kernel_data| {
                                kernel_data.schedule_upcall(upcall::NEW_REGISTRATION, (0, 0, 0))
                            });
                        }
                    }
                }
                Ok(())
            })
    }

    fn discover(&self, processid: ProcessId) -> Result<(), ErrorCode> {
        // Get allowed name to compare
        let mut this_name: [u8; 20] = [0; 20];
        self.apps
            .enter(processid, |_, this_kerneldata| {
                this_kerneldata
                    .get_readonly_processbuffer(ro_allow::STRING_NAME)
                    .map(|allow_name| {
                        allow_name
                            .enter(|buf| {
                                if buf.len() != 20 {
                                    // Error if allowed name is not exactly 20 bytes
                                    Err(ErrorCode::SIZE)
                                } else {
                                    let n = core::cmp::min(buf.len(), this_name.len());
                                    buf[0..n].copy_to_slice(&mut this_name[0..n]);
                                    Ok(())
                                }
                            })
                            .unwrap_or_else(|err| err.into())
                    })
                    .unwrap_or_else(|err| err.into())
            })
            .unwrap_or_else(|err| err.into())?;

        // Cannot check for empty name, as that is the default value and could
        // match processes that haven't registered
        if this_name == [0; 20] {
            return Err(ErrorCode::INVAL);
        }

        // Check for matching names
        for cntr in self.apps.iter() {
            if cntr.processid() != processid {
                let otherid = cntr.processid();

                if cntr.enter(|other_app, _| this_name == other_app.registered_name) {
                    // Found matching app!

                    // If discovery validation is desired, this is where it
                    // would occur before scheduling the upcall

                    // Schedule discovery complete callback
                    let _ = self.apps.enter(processid, |_, kernel_data| {
                        kernel_data
                            .schedule_upcall(upcall::DISCOVERY_COMPLETE, (1, otherid.id(), 0))
                    });

                    return Ok(());
                }
            }
        }
        Err(ErrorCode::UNINSTALLED)
    }

    fn enable_registration_notifications(&self, processid: ProcessId) -> Result<(), ErrorCode> {
        self.apps
            .enter(processid, |app, _| {
                app.get_registration_notifications = true;
                Ok(())
            })
            .unwrap_or_else(|err| err.into())
    }

    fn disable_registration_notifications(&self, processid: ProcessId) -> Result<(), ErrorCode> {
        self.apps
            .enter(processid, |app, _| {
                app.get_registration_notifications = false;
                Ok(())
            })
            .unwrap_or_else(|err| err.into())
    }
}

impl SyscallDriver for IpcRegistryStringName {
    /// Registration and discovery of IPC services
    ///
    /// Matches based on "names": length 20 arrays of u8. Typically UTF-8
    /// strings (without null-termination), but no explicit requirement of
    /// format.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Check driver presence
    /// - `1`: Register as service with allowed name
    /// - `2`: Discover service with allowed name
    /// - `3`: Get notifications when a service registers
    /// - `4`: Stop notifications when a service registers
    fn command(
        &self,
        command_num: usize,
        _: usize,
        _: usize,
        processid: ProcessId,
    ) -> CommandReturn {
        match command_num {
            0 => CommandReturn::success(),
            1 => self.register(processid).into(),
            2 => self.discover(processid).into(),
            3 => self.enable_registration_notifications(processid).into(),
            4 => self.disable_registration_notifications(processid).into(),
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}
