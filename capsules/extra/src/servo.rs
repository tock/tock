// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use kernel::hil;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use capsules_core::driver;
pub const DRIVER_NUM: usize = driver::NUM::Servo as usize;

#[derive(Clone, Copy, PartialEq)]
pub enum ServoCommand {
    Servo { angle: usize },
}

pub struct Servo<'a, B: hil::servo::Servo<'a>> {
    /// The service capsule servo.
    servo: &'a B,
}

impl<'a, B: hil::servo::Servo<'a>> Servo<'a, B> {
    pub fn new(servo: &'a B) -> Servo<'a, B> {
        Servo { servo: servo }
    }
}
/// Provide an interface for userland.
impl<'a, B: hil::servo::Servo<'a>> SyscallDriver for Servo<'a, B> {
    /// Command interface.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Return Ok(()) if this driver is included on the platform.
    /// - `1`: Changing the angle immediatelly. `data1` is used for the angle (0-180).
    fn command(
        &self,
        command_num: usize,
        data1: usize,
        _data2: usize,
        _processid: ProcessId,
    ) -> CommandReturn {
        match command_num {
            // Check whether the driver exists.
            0 => CommandReturn::success(),
            // Change the angle immediately.
            1 => self.servo.servo(data1).into(),
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, _process_id: ProcessId) -> Result<(), kernel::process::Error> {
        todo!()
    }
}
