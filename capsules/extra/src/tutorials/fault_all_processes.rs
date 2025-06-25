// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Driver for the hardware root of trust tutorial, to fault all apps when
//! requested from a user application.

use kernel::capabilities::ProcessManagementCapability;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::{ErrorCode, Kernel, ProcessId};

pub struct FaultAllProcesses<C: ProcessManagementCapability> {
    kernel: &'static Kernel,
    capability: C,
}

impl<C: ProcessManagementCapability> FaultAllProcesses<C> {
    pub fn new(kernel: &'static Kernel, capability: C) -> Self {
        FaultAllProcesses { kernel, capability }
    }
}

impl<C: ProcessManagementCapability> SyscallDriver for FaultAllProcesses<C> {
    fn command(&self, command_num: usize, _: usize, _: usize, _: ProcessId) -> CommandReturn {
        match command_num {
            0 => CommandReturn::success(),
            1 => {
                kernel::debug!("Hardfaulting all applications...");
                self.kernel.hardfault_all_apps(&self.capability);
                kernel::debug!("All applications hardfaulted.");
                CommandReturn::success()
            }
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, _: ProcessId) -> Result<(), kernel::process::Error> {
        Ok(())
    }
}
