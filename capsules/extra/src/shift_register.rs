// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//!  Userspace Interface for SR74HC shift register
//!
//!  ### `command_number`
//!
//!  - '0': writes a number. Returns ErrorCode FAIL if the value is larger than the maximum size
//!  - '1': returns the number of bits
//!

use kernel::hil::shift_register::ShiftRegister;

use kernel::ErrorCode;
use kernel::process::{Error, ProcessId};
use kernel::syscall::{CommandReturn, SyscallDriver};

use capsules_core::driver;
pub const DRIVER_NUM: usize = driver::NUM::ShiftRegister as usize;

pub struct ShiftRegisterDriver<'a, Sr: ShiftRegister> {
    shift_register: &'a Sr,
}

impl<'a, Sr: ShiftRegister> ShiftRegisterDriver<'a, Sr> {
    pub fn new(shift_register: &'a Sr) -> Self {
        Self { shift_register }
    }
}

impl<Sr: ShiftRegister> SyscallDriver for ShiftRegisterDriver<'_, Sr> {
    fn command(
        &self,
        command_number: usize,
        r2: usize,
        _r3: usize,
        _process_id: ProcessId,
    ) -> CommandReturn {
        match command_number {
            0 => CommandReturn::success(),

            1 => {
                if r2 >= (1 << Sr::BITS) {
                    return CommandReturn::failure(ErrorCode::SIZE);
                }
                let rez = self.shift_register.write(r2);
                CommandReturn::success()
            }

            2 => CommandReturn::success_u32(Sr::BITS as u32),

            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, _process_id: ProcessId) -> Result<(), Error> {
        Ok(())
    }
}
