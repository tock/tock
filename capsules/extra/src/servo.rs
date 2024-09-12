// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! This provides virtualized userspace access to a servomotor.
//!
//! Usage
//! -----
//!
//! ```rust,ignore
//! # use kernel::static_init;
//!     let mux_pwm = components::pwm::PwmMuxComponent::new(&peripherals.pwm)
//!         .finalize(components::pwm_mux_component_static!(rp2040::pwm::Pwm));
//!
//!     let virtual_pwm_servo =
//!        components::pwm::PwmPinUserComponent::new(mux_pwm, rp2040::gpio::RPGpio::GPIO4)
//!             .finalize(components::pwm_pin_user_component_static!(rp2040::pwm::Pwm));
//!
//!     let pwm_servo = static_init!(
//!         capsules_extra::sg90::Sg90<
//!             'static,
//!             capsules_core::virtualizers::virtual_pwm::PwmPinUser<'static, rp2040::pwm::Pwm>,
//!         >,
//!         capsules_extra::sg90::Sg90::new(
//!             virtual_pwm_servo,
//!         )
//!     );
//!
//!     let servo_driver = static_init!(
//!         capsules_extra::servo::Servo<
//!             'static,
//!             capsules_extra::sg90::Sg90<
//!                 'static,
//!                 capsules_core::virtualizers::virtual_pwm::PwmPinUser<'static, rp2040::pwm::Pwm>,
//!             >,
//!         >,
//!         capsules_extra::servo::Servo::new(pwm_servo,)
//!     );

use kernel::hil;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use capsules_core::driver;
pub const DRIVER_NUM: usize = driver::NUM::Servo as usize;

pub struct Servo<'a, const NUM_SERVO: usize> {
    /// The service capsule servo.
    servo: &'a [&'a dyn hil::servo::Servo<'a>; NUM_SERVO],
}

impl<'a, const NUM_SERVO: usize> Servo<'a, NUM_SERVO> {
    pub fn new(servo: &'a [&'a dyn hil::servo::Servo<'a>; NUM_SERVO]) -> Self {
        for servo in servo.iter() {
            let _ = servo.set_angle(0);
        }
        Self { servo }
    }
}
/// Provide an interface for userland.
impl<'a, const NUM_SERVO: usize> SyscallDriver for Servo<'a, NUM_SERVO> {
    /// Command interface.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Return Ok(()) if this driver is included on the platform.
    /// - `1`: Changing the angle immediatelly. `data1` is used for the angle (0-180).
    /// - `2`: Returning the current angle.
    fn command(
        &self,
        command_num: usize,
        servo_index: usize,
        angle: usize,
        _processid: ProcessId,
    ) -> CommandReturn {
        match command_num {
            // Check whether the driver exists.
            0 => CommandReturn::success(),
            // Change the angle immediately.
            1 => {
                if servo_index >= NUM_SERVO {
                    CommandReturn::failure(ErrorCode::INVAL)
                } else {
                    match angle.try_into() {
                        Ok(angle) => match self.servo[servo_index].set_angle(angle) {
                            Ok(()) => CommandReturn::success(),
                            Err(err) => CommandReturn::failure(err),
                        },
                        Err(_) => CommandReturn::failure(ErrorCode::INVAL),
                    }
                }
            }
            // Return the current angle.
            2 => match self.servo[servo_index].get_angle() {
                Ok(angle) => CommandReturn::success_u32(angle as u32),
                Err(err) => CommandReturn::failure(err),
            },
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, _process_id: ProcessId) -> Result<(), kernel::process::Error> {
        Ok(())
    }
}
