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
//! Usage
//! -----
//!
//! ```rust,ignore
//! TODO
//! pub struct PMCapability;
//! unsafe impl capabilities::ProcessManagementCapability for PMCapability {}
//!
//! let ipc_registry_package_name = components::ipc::ipc_registry_package_name::IpcRegistryPackageNameComponent::new(
//!     board_kernel,
//!     capsules_core::ipc::ipc_registry_package_name::DRIVER_NUM,
//!     PMCapability,
//!     create_capability!(capabilities::MemoryAllocationCapability),
//! ).finalize(components::ipc_registry_package_name_component_static!(PMCapability));
//! ```

use crate::ipc::ipc_identifier::IpcIdentifier;
use kernel::capabilities::ProcessManagementCapability;
use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::processbuffer::ReadableProcessBuffer;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::{ErrorCode, Kernel, ProcessId};

/// Syscall driver number.
pub const DRIVER_NUM: usize = crate::driver::NUM::IpcRegistryPackageName as usize;

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
    /// Number of upcalls.
    pub const COUNT: u8 = 2;
}

/// Per-process metadata
#[derive(Default)]
pub struct App {
    is_registered: bool,
}

/// Validation function signature
///
/// Arguments:
///  * Process to be validated
///  * Name it is attempting to register with
///  * Function to call to complete validation which itself takes one argument:
///    a boolean for whether registration is allowed
///
/// Return:
///  * Result, which will be an error if validation cannot be performed
//pub type ValidationFunction = fn(ProcessId, &[u8], F) -> Result<(), ErrorCode> where F: FnOnce();
pub type ValidationFunction<C> =
    fn(ProcessId, &[u8], &IpcRegistryPackageName<C>) -> Result<(), ErrorCode>;

/// IPC Registry Package Name capsule
///
/// This capsule allows for registration and discovery of IPC processes via the
/// Package Name field of their TBF header.
pub struct IpcRegistryPackageName<C: ProcessManagementCapability> {
    /// Grant memory
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

    /// Optional validation function. If this function is supplied it will be
    /// called to determine if validation can succeed.
    //validation: Option<fn(ProcessId, &[u8], &IpcRegistryPackageName<C>, &dyn Fn(&IpcRegistryPackageName<C>, ProcessId, bool)) -> Result<(), ErrorCode>>,
    // validation: Option<fn(ProcessId, &[u8], &dyn FnOnce(bool)) -> Result<(), ErrorCode>>,
    validation: Option<ValidationFunction<C>>,
}

impl<C: ProcessManagementCapability> IpcRegistryPackageName<C> {
    /// Create a new IpcRegistryPackageName capsule
    pub fn new(
        grant: Grant<
            App,
            UpcallCount<{ upcall::COUNT }>,
            AllowRoCount<{ ro_allow::COUNT }>,
            AllowRwCount<0>,
        >,
        kernel: &'static Kernel,
        capability: C,
        //validation: Option<fn(ProcessId, &[u8], &dyn FnOnce(bool)) -> Result<(), ErrorCode>>,
        validation: Option<ValidationFunction<C>>,
    ) -> Self {
        Self {
            apps: grant,
            kernel,
            capability,
            validation,
        }
    }

    pub fn complete_registration(&self, processid: ProcessId, registration_allowed: bool) {
        // Save registration state and upcall result. We're going to have to
        // assume this works, as we can't signal issues to the process anymore
        // at this point.
        let _ = self.apps.enter(processid, |app, kerneldata| {
            if registration_allowed {
                app.is_registered = true;

                // Schedule registration complete callback with success
                // upcall arguments-> status: StatusCode
                let _ = kerneldata.schedule_upcall(upcall::REGISTRATION_COMPLETE, (0, 0, 0));
            } else {
                app.is_registered = false;

                // Schedule registration complete callback with failure
                // upcall arguments-> status: StatusCode
                let _ = kerneldata.schedule_upcall(
                    upcall::REGISTRATION_COMPLETE,
                    (ErrorCode::FAIL.into(), 0, 0),
                );
            }
        });
    }

    fn register(&self, processid: ProcessId) -> Result<(), ErrorCode> {
        // Check for valid package name and validate process
        self.kernel.process_map_or_external(
            Err(ErrorCode::NOMEM),
            processid,
            |process| {
                if process.get_process_name() == "" {
                    // Invalid Package Name
                    Err(ErrorCode::NOMEM)
                } else {
                    // Valid package name
                    if let Some(validator) = self.validation {
                        // Call validation function, which must invoke our closure with true for success or false for failure
                        validator(processid, process.get_process_name().as_bytes(), self)
                    } else {
                        // No validation installed. Complete registration now
                        self.complete_registration(processid, true);
                        Ok(())
                    }
                }
            },
            &self.capability,
        )
    }

    fn compare_names(&self, clientid: ProcessId, serverid: ProcessId) -> bool {
        // Compare a server package name and client allowed buffer
        // If any errors occur, returns false
        self.apps
            .enter(clientid, |_, kerneldata| {
                kerneldata
                    .get_readonly_processbuffer(ro_allow::NAME)
                    .map(|allow_name| {
                        allow_name.enter(|buf| {
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
                    })
            })
            .flatten()
            .flatten()
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
                        self.apps.enter(processid, |_, kerneldata| {
                            let ipc_id = IpcIdentifier::new_from_processid(otherid);
                            // upcall arguments-> status: StatusCode, ipc_id_lower: u32, ipc_id_upper: u32
                            let _ = kerneldata.schedule_upcall(
                                upcall::DISCOVERY_COMPLETE,
                                (0, ipc_id.lower() as usize, ipc_id.upper() as usize),
                            );
                        })?;

                        // There won't be another match, so return early
                        return Ok(());
                    }
                }
            }
        }

        // No match found, return successfully but upcall that discovery failed instead
        self.apps.enter(processid, |_, kerneldata| {
            // upcall arguments-> status: StatusCode
            let _ = kerneldata.schedule_upcall(
                upcall::DISCOVERY_COMPLETE,
                (ErrorCode::NODEVICE.into(), 0, 0),
            );
        })?;
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
