// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

use kernel::errorcode;
use kernel::platform::SyscallFilter;
use kernel::process;
use kernel::process::ShortId;
use kernel::syscall;

pub struct DynamicPoliciesCustomFilter {}

impl SyscallFilter for DynamicPoliciesCustomFilter {
    fn filter_syscall(
        &self,
        process: &dyn process::Process,
        _syscall: &syscall::Syscall,
    ) -> Result<(), errorcode::ErrorCode> {
        // Get the upper four bits of the ShortId.
        let signing_key_id = if let ShortId::Fixed(fixed_id) = process.short_app_id() {
            ((u32::from(fixed_id) >> 28) & 0xF) as u8
        } else {
            0xff_u8
        };

        // Enforce the correct policy based on the signing key and the system
        // call.
        //
        // Documentation for system call:
        // https://docs.tockos.org/kernel/syscall/enum.syscall#implementations
        match signing_key_id {
            0 => Ok(()),
            1 => Ok(()),
            _ => Ok(()),
        }
    }
}
