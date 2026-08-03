// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Capsule implementing the IPC Registry mechanism with string names.
//!
//! This capsule allows services to register with arbitrary 20-byte values,
//! typically strings. Capsules can discover them using those same 20-byte
//! values.
//!
//! Usage
//! -----
//!
//! ```rust,ignore
//! let ipc_registry_string_name = components::ipc::ipc_registry_string_name::IpcRegistryStringNameComponent::new(
//!     board_kernel,
//!     capsules_core::ipc::ipc_registry_string_name::DRIVER_NUM,
//!     create_capability!(capabilities::MemoryAllocationCapability),
//! ).finalize(components::ipc_registry_string_name_component_static!());
//! ```

use crate::ipc::ipc_identifier::IpcIdentifier;
use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::processbuffer::ReadableProcessBuffer;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
pub const DRIVER_NUM: usize = crate::driver::NUM::IpcRegistryStringName as usize;

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
    /// Number of upcalls.
    pub const COUNT: u8 = 2;
}

/// Maximum string length, with a value of 20 by default.
const MAX_STRING_LEN: usize = 20;

/// Per-process metadata
#[derive(Default)]
pub struct App {
    // Defaults to an array of all-zero values
    registered_name: [u8; MAX_STRING_LEN],
}

/// IPC Registry String Name capsule
///
/// This capsule allows for registration and discovery of IPC processes via a
/// string provided by the process.
pub struct IpcRegistryStringName {
    /// Grant memory
    apps: Grant<
        App,
        UpcallCount<{ upcall::COUNT }>,
        AllowRoCount<{ ro_allow::COUNT }>,
        AllowRwCount<0>,
    >,
}

impl IpcRegistryStringName {
    // Create a new IPC Registry String Name capsule
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
        // comparing or saving the name itself

        // Get allowed name to validate and later save
        let mut new_name: [u8; MAX_STRING_LEN] = [0; MAX_STRING_LEN];
        self.apps.enter(processid, |_, kerneldata| {
            kerneldata
                .get_readonly_processbuffer(ro_allow::STRING_NAME)
                .and_then(|allow_name| {
                    allow_name.enter(|buf| {
                        if buf.len() != MAX_STRING_LEN {
                            // Error if allowed name is not exactly MAX_STRING_LEN bytes
                            Err(ErrorCode::SIZE)
                        } else {
                            let n = core::cmp::min(buf.len(), new_name.len());
                            buf[0..n].copy_to_slice(&mut new_name[0..n]);
                            Ok(())
                        }
                    })
                })
        })???;

        // Cannot register an empty name, as that is the default value
        if new_name == [0; MAX_STRING_LEN] {
            return Err(ErrorCode::INVAL);
        }

        // Check for matching names in already-registered apps
        for cntr in self.apps.iter() {
            if cntr.processid() != processid
                && cntr.enter(|other_app, _| new_name == other_app.registered_name)
            {
                // Found matching app!

                // We really can't have two apps with the same name, so
                // we'll give an error to this second app. First-come-first-served.
                return Err(ErrorCode::ALREADY);
            }
        }

        // Save newly registered name
        self.apps.enter(processid, |app, kerneldata| {
            // Copy name into grant space
            let n = core::cmp::min(new_name.len(), app.registered_name.len());
            app.registered_name[0..n].copy_from_slice(&new_name[0..n]);

            // Schedule registration complete callback
            // upcall arguments-> status: StatusCode
            let _ = kerneldata.schedule_upcall(upcall::REGISTRATION_COMPLETE, (0, 0, 0));
        })?;

        Ok(())
    }

    fn discover(&self, processid: ProcessId) -> Result<(), ErrorCode> {
        // Get allowed name to compare
        let mut this_name: [u8; MAX_STRING_LEN] = [0; MAX_STRING_LEN];
        self.apps.enter(processid, |_, kerneldata| {
            kerneldata
                .get_readonly_processbuffer(ro_allow::STRING_NAME)
                .and_then(|allow_name| {
                    allow_name.enter(|buf| {
                        if buf.len() != MAX_STRING_LEN {
                            // Error if allowed name is not exactly MAX_STRING_LEN bytes
                            Err(ErrorCode::SIZE)
                        } else {
                            let n = core::cmp::min(buf.len(), this_name.len());
                            buf[0..n].copy_to_slice(&mut this_name[0..n]);
                            Ok(())
                        }
                    })
                })
        })???;

        // Cannot check for empty name, as that is the default value and could
        // match processes that haven't registered
        if this_name == [0; MAX_STRING_LEN] {
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
                    self.apps.enter(processid, |_, kerneldata| {
                        let ipc_id = IpcIdentifier::new_from_processid(otherid);
                        // upcall arguments-> status: StatusCode, ipc_id_lower: u32, ipc_id_upper: u32
                        let _ = kerneldata.schedule_upcall(
                            upcall::DISCOVERY_COMPLETE,
                            (0, ipc_id.lower() as usize, ipc_id.upper() as usize),
                        );
                    })?;

                    // Discovery complete
                    return Ok(());
                }
            }
        }

        // No match found, return successfully but upcall that discovery failed
        let _ = self.apps.enter(processid, |_, kerneldata| {
            // upcall arguments-> status: StatusCode, ipc_id_lower: u32, ipc_id_upper: u32
            let _ = kerneldata.schedule_upcall(
                upcall::DISCOVERY_COMPLETE,
                (ErrorCode::NODEVICE.into(), 0, 0),
            );
        });
        Ok(())
    }
}

impl SyscallDriver for IpcRegistryStringName {
    /// Registration and discovery of IPC services
    ///
    /// Matches based on "names": length MAX_STRING_LEN arrays of u8.
    /// Typically UTF-8 strings (without null-termination), but no explicit
    /// requirement of format.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Check driver presence
    /// - `1`: Register as service with allowed name
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
