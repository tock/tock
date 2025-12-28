// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Capsule implementing the IPC Registry mechanism with package names.
//!
//! This capsule allows services to register the package name field from their
//! TBF header. These are UTF-8 formatted strings of arbitrary length.
//! https://book.tockos.org/doc/tock_binary_format#3-package-name
//! Capsules can discover services by allowing a matching UTF-8 string.
//!
//! This capsule requires a ProcessManagementCapability to view process names.
//!
//! TODO add example of how to instantiate

use kernel::capabilities::ProcessManagementCapability;
use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::processbuffer::ReadableProcessBuffer;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::{ErrorCode, Kernel, ProcessId};

/// Syscall driver number.
use capsules_core::driver;
pub const DRIVER_NUM: usize = driver::NUM::IpcRegistryPackageName as usize;

/// Ids for read-only allow buffers
mod ro_allow {
    pub const NAME: usize = 0;
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
    is_registered: bool,
}

pub struct IpcRegistryPackageName<C: ProcessManagementCapability> {
    apps: Grant<
        App,
        UpcallCount<{ upcall::COUNT }>,
        AllowRoCount<{ ro_allow::COUNT }>,
        AllowRwCount<0>,
    >,

    /// Reference to the kernel object so we can access process state.
    kernel: &'static Kernel,

    /// This capsule needs to use potentially dangerous APIs related to
    /// processes, and requires a capability to access those APIs.
    capability: C,
}

impl<C: ProcessManagementCapability> IpcRegistryPackageName<C> {
    pub fn new(
        grant: Grant<
            App,
            UpcallCount<{ upcall::COUNT }>,
            AllowRoCount<{ ro_allow::COUNT }>,
            AllowRwCount<0>,
        >,
        kernel: &'static Kernel,
        capability: C,
    ) -> Self {
        Self {
            apps: grant,
            kernel,
            capability,
        }
    }

    fn register(&self, processid: ProcessId) -> Result<(), ErrorCode> {
        // If registration validation is desired, that would go here before
        // saving the name

        // Ensure that a package name field exists
        if !self.kernel.process_map_or_external(
            false,
            processid,
            |process| process.get_process_name() != "",
            &self.capability,
        ) {
            return Err(ErrorCode::NOMEM);
        }

        // Save registration state
        self.apps
            .enter(processid, |app, kerneldata| {
                app.is_registered = true;

                // Schedule registration complete callback
                let _ = kerneldata.schedule_upcall(upcall::REGISTRATION_COMPLETE, (1, 0, 0));
                Ok(())
            })
            .unwrap_or_else(|err| err.into())
            .map(|()| {
                // Notify all other apps of a new registration. Only apps that are subscribed will get the notification.
                self.apps.each(|otherid, _, kerneldata| {
                    if otherid != processid {
                        let _ = kerneldata.schedule_upcall(upcall::NEW_REGISTRATION, (0, 0, 0));
                    }
                });
            })
    }

    fn compare_names(&self, clientid: ProcessId, serverid: ProcessId) -> bool {
        // Compare a server package name and client allowed buffer
        // If any errors occur, returns false
        self.apps
            .enter(clientid, |_, this_kerneldata| {
                this_kerneldata
                    .get_readonly_processbuffer(ro_allow::NAME)
                    .map(|allow_name| {
                        allow_name
                            .enter(|buf| {
                                self.kernel.process_map_or_external(
                                    false,
                                    serverid,
                                    |server| {
                                        let package_name = server.get_process_name().as_bytes();

                                        // Compare TBF header package name with user-provided name, byte-by-byte
                                        package_name.len() == buf.len()
                                            && package_name
                                                .iter()
                                                .zip(buf.iter())
                                                .all(|(c1, c2)| *c1 == c2.get())
                                    },
                                    &self.capability,
                                )
                            })
                            .unwrap_or(false)
                    })
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    }

    fn discover(&self, processid: ProcessId) -> Result<(), ErrorCode> {
        // Iterate registered services
        for cntr in self.apps.iter() {
            if cntr.processid() != processid {
                let otherid = cntr.processid();

                if cntr.enter(|other_app, _| other_app.is_registered) {
                    // Found a registered service

                    // Check if it matches
                    if self.compare_names(processid, otherid) {
                        // Found a matching service!

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
        }

        // No match found, return successfully but upcall that discovery failed
        let _ = self.apps.enter(processid, |_, kernel_data| {
            kernel_data.schedule_upcall(upcall::DISCOVERY_COMPLETE, (0, 0, 0))
        });
        Ok(())
    }
}

impl<C: ProcessManagementCapability> SyscallDriver for IpcRegistryPackageName<C> {
    /// Registration and discovery of IPC services
    ///
    /// Matches based on server package name and client allowed buffer.
    /// Both are formatted in UTF-8 with no particular length constraints.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Check driver presence
    /// - `1`: Register as service using package name
    /// - `2`: Discover service with allowed name
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
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}
