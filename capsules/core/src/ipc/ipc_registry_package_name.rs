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
//! pub struct PMCapability;
//! unsafe impl capabilities::ProcessManagementCapability for PMCapability {}
//!
//! let ipc_registry_package_name = components::ipc::ipc_registry_package_name::IpcRegistryPackageNameComponent::new(
//!     board_kernel,
//!     capsules_core::ipc::ipc_registry_package_name::DRIVER_NUM,
//!     &capsules_core::ipc::filters::IpcPackageNameRegistrationFilterNull {},
//!     PMCapability,
//!     create_capability!(capabilities::MemoryAllocationCapability),
//! ).finalize(components::ipc_registry_package_name_component_static!(PMCapability));
//! ```

use crate::ipc::ipc_identifier::IpcIdentifier;
use kernel::capabilities::ProcessManagementCapability;
use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::platform::registration::RegistrationFilter;
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

/// IPC Registry Package Name capsule
///
/// This capsule allows for registration and discovery of IPC processes via the
/// Package Name field of their TBF header.
pub struct IpcRegistryPackageName<'a, RF: RegistrationFilter, C: ProcessManagementCapability> {
    /// Grant memory
    apps: Grant<
        App,
        UpcallCount<{ upcall::COUNT }>,
        AllowRoCount<{ ro_allow::COUNT }>,
        AllowRwCount<0>,
    >,

    /// Filter for validating service registrations.
    registration_filter: &'a RF,

    /// Reference to the kernel object so we can access process state.
    kernel: &'static Kernel,

    /// This capsule needs to use potentially dangerous APIs related to
    /// processes, and requires a capability to access those APIs.
    capability: C,
}

impl<
    'a,
    RF: RegistrationFilter<RegistrationIdentifier = &'static str>,
    C: ProcessManagementCapability,
> IpcRegistryPackageName<'a, RF, C>
{
    /// Create a new IpcRegistryPackageName capsule
    pub fn new(
        grant: Grant<
            App,
            UpcallCount<{ upcall::COUNT }>,
            AllowRoCount<{ ro_allow::COUNT }>,
            AllowRwCount<0>,
        >,
        registration_filter: &'a RF,
        kernel: &'static Kernel,
        capability: C,
    ) -> Self {
        Self {
            apps: grant,
            registration_filter,
            kernel,
            capability,
        }
    }

    fn register(&self, processid: ProcessId) -> Result<(), ErrorCode> {
        // Check that package name exists, and validate it
        self.kernel.process_map_or_external(
            Err(ErrorCode::NOMEM),
            processid,
            |process| {
                if process.get_process_name() == "" {
                    // Can't register without a package name
                    Err(ErrorCode::NOMEM)
                } else {
                    // Validate this registration attempt
                    self.registration_filter
                        .filter_registration(processid, &process.get_process_name())
                }
            },
            &self.capability,
        )?;

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
        self.apps.enter(processid, |app, kerneldata| {
            app.is_registered = true;

            // Schedule registration complete callback
            // upcall arguments-> status: StatusCode
            let _ = kerneldata.schedule_upcall(upcall::REGISTRATION_COMPLETE, (0, 0, 0));
        })?;

        Ok(())
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

impl<RF: RegistrationFilter<RegistrationIdentifier = &'static str>, C: ProcessManagementCapability>
    SyscallDriver for IpcRegistryPackageName<'_, RF, C>
{
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
