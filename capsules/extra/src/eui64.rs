// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Provides an EUI-64 (Extended Unique Identifier) interface for userspace.

use capsules_core::driver;
pub const DRIVER_NUM: usize = driver::NUM::Eui64 as usize;

use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::{ErrorCode, ProcessId};

pub struct Eui64 {
    eui64: u64,
}

impl Eui64 {
    pub fn new(eui64: u64) -> Eui64 {
        Eui64 { eui64 }
    }
}

impl SyscallDriver for Eui64 {
    /// Control the Eui64.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Driver existence check.
    /// - `1`: Obtain EUI64 - providing the value within a u64 returncode.
    fn command(&self, command_num: usize, _: usize, _: usize, _: ProcessId) -> CommandReturn {
        match command_num {
            0 => CommandReturn::success(),
            1 => CommandReturn::success_u64(self.eui64),
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, _: ProcessId) -> Result<(), kernel::process::Error> {
        Ok(())
    }
}
