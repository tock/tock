// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Implementations of [`SyscallFilter`] using TBF headers.

use kernel::errorcode;
use kernel::platform::SyscallFilter;
use kernel::process;
use kernel::syscall;
use tock_tbf::types::CommandPermissions;

/// An allow list system call filter based on the TBF header, with a default
/// allow all fallback.
///
/// This will check if the process has TbfHeaderPermissions specified. If the
/// process has TbfHeaderPermissions they will be used to determine access
/// permissions. For details on this see the TockBinaryFormat documentation. If
/// no permissions are specified the default is to allow the syscall.
pub struct TbfHeaderFilterDefaultAllow {}

/// Implement default SyscallFilter trait for filtering based on the TBF header.
impl SyscallFilter for TbfHeaderFilterDefaultAllow {
    fn filter_syscall(
        &self,
        process: &dyn process::Process,
        syscall: &syscall::Syscall,
    ) -> Result<(), errorcode::ErrorCode> {
        match syscall {
            // Subscribe is allowed if any commands are
            syscall::Syscall::Subscribe {
                driver_number,
                subdriver_number: _,
                upcall_ptr: _,
                appdata: _,
            } => match process.get_command_permissions(*driver_number, 0) {
                CommandPermissions::NoPermsAtAll => Ok(()),
                CommandPermissions::NoPermsThisDriver => Err(errorcode::ErrorCode::NODEVICE),
                CommandPermissions::Mask(_allowed) => Ok(()),
            },

            syscall::Syscall::Command {
                driver_number,
                subdriver_number,
                arg0: _,
                arg1: _,
            } => match process.get_command_permissions(*driver_number, subdriver_number / 64) {
                CommandPermissions::NoPermsAtAll => Ok(()),
                CommandPermissions::NoPermsThisDriver => Err(errorcode::ErrorCode::NODEVICE),
                CommandPermissions::Mask(allowed) => {
                    if (1 << (subdriver_number % 64)) & allowed > 0 {
                        Ok(())
                    } else {
                        Err(errorcode::ErrorCode::NODEVICE)
                    }
                }
            },

            // Allow is allowed if any commands are
            syscall::Syscall::ReadWriteAllow {
                driver_number,
                subdriver_number: _,
                allow_address: _,
                allow_size: _,
            } => match process.get_command_permissions(*driver_number, 0) {
                CommandPermissions::NoPermsAtAll => Ok(()),
                CommandPermissions::NoPermsThisDriver => Err(errorcode::ErrorCode::NODEVICE),
                CommandPermissions::Mask(_allowed) => Ok(()),
            },

            // Allow is allowed if any commands are
            syscall::Syscall::UserspaceReadableAllow {
                driver_number,
                subdriver_number: _,
                allow_address: _,
                allow_size: _,
            } => match process.get_command_permissions(*driver_number, 0) {
                CommandPermissions::NoPermsAtAll => Ok(()),
                CommandPermissions::NoPermsThisDriver => Err(errorcode::ErrorCode::NODEVICE),
                CommandPermissions::Mask(_allowed) => Ok(()),
            },

            // Allow is allowed if any commands are
            syscall::Syscall::ReadOnlyAllow {
                driver_number,
                subdriver_number: _,
                allow_address: _,
                allow_size: _,
            } => match process.get_command_permissions(*driver_number, 0) {
                CommandPermissions::NoPermsAtAll => Ok(()),
                CommandPermissions::NoPermsThisDriver => Err(errorcode::ErrorCode::NODEVICE),
                CommandPermissions::Mask(_allowed) => Ok(()),
            },

            // Non-filterable system calls
            syscall::Syscall::Yield { .. }
            | syscall::Syscall::Memop { .. }
            | syscall::Syscall::Exit { .. } => Ok(()),
        }
    }
}
