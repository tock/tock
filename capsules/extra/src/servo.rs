// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! This provides virtualized userspace access to a servomotor.
//!
//! Usage
//! -----
//!
//! ```ignore
//! use kernel::static_init;
//! let mux_pwm = components::pwm::PwmMuxComponent::new(&peripherals.pwm)
//! .finalize(components::pwm_mux_component_static!(rp2040::pwm::Pwm));
//!
//! let virtual_pwm_servo: &PwmPinUser<'static, rp2040::pwm::Pwm<'static>> =
//! components::pwm::PwmPinUserComponent::new(mux_pwm, rp2040::gpio::RPGpio::GPIO4)
//!     .finalize(components::pwm_pin_user_component_static!(rp2040::pwm::Pwm));
//!
//! let sg90_servo = static_init!(
//! capsules_extra::sg90::Sg90<
//!    'static,
//!    capsules_core::virtualizers::virtual_pwm::PwmPinUser<'static, rp2040::pwm::Pwm>,
//! >,
//! capsules_extra::sg90::Sg90::new(virtual_pwm_servo)
//! );
//!
//! // Here, we initialize an array of two SG90 servomotors as an example.
//! let multi_servo = static_init!(
//! [&'static dyn hil::servo::Servo<'static>; 2],
//! [sg90_servo, sg90_servo]
//! );
//! let servo = static_init!(
//! capsules_extra::servo::Servo<'static, 2>,
//! capsules_extra::servo::Servo::new(multi_servo)
//! );
//! ```

use kernel::hil;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use capsules_core::driver;
pub const DRIVER_NUM: usize = driver::NUM::Servo as usize;

pub struct Servo<'a, const SERVO_COUNT: usize> {
    /// The service capsule servo.
    servo: &'a [&'a dyn hil::servo::Servo<'a>; SERVO_COUNT],
}

impl<'a, const SERVO_COUNT: usize> Servo<'a, SERVO_COUNT> {
    pub fn new(servo: &'a [&'a dyn hil::servo::Servo<'a>; SERVO_COUNT]) -> Self {
        Self { servo }
    }
}
/// Provide an interface for userland.
impl<const SERVO_COUNT: usize> SyscallDriver for Servo<'_, SERVO_COUNT> {
    /// Command interface.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Return Ok(()) if this driver is included on the platform.
    /// - `1`: Returns an u32 representing the number of available servomotors.
    /// - `2`: Changing the angle immediatelly.`servo_index` receives the index
    /// corresponding to the servo whose angle we want to adjust
    /// `angle` is used to receive a value between 0 and 180.
    /// - `3`: Returning the current angle for a specific index.
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
            // Returns the number of available servomotors.
            1 => CommandReturn::success_u32(SERVO_COUNT as u32),
            // Change the angle immediately.
            2 => {
                if servo_index >= SERVO_COUNT {
                    CommandReturn::failure(ErrorCode::NODEVICE)
                } else {
                    match angle.try_into() {
                        Ok(angle) => match self.servo[servo_index].set_angle(angle) {
                            Ok(()) => CommandReturn::success(),
                            Err(_) => CommandReturn::failure(ErrorCode::FAIL),
                        },
                        Err(_) => CommandReturn::failure(ErrorCode::INVAL),
                    }
                }
            }
            // Return the current angle.
            3 => {
                if servo_index >= SERVO_COUNT {
                    CommandReturn::failure(ErrorCode::NODEVICE)
                } else {
                    match self.servo[servo_index].get_angle() {
                        Ok(angle) => CommandReturn::success_u32(angle as u32),
                        Err(err) => CommandReturn::failure(err),
                    }
                }
            }
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, _process_id: ProcessId) -> Result<(), kernel::process::Error> {
        Ok(())
    }
}
