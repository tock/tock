// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! This provides virtualized userspace access to a buzzer.
//!
//! Each app can have one outstanding buzz request, and buzz requests will queue
//! with each app getting exclusive access to the buzzer during its turn. Apps
//! can specify the frequency and duration of the square wave buzz, but the
//! duration is capped to prevent this from being annoying.
//!
//! Apps can subscribe to an optional callback if they care about getting
//! buzz done events.
//!
//! Usage
//! -----
//!
//! ```rust,ignore
//! # use kernel::static_init;
//! let mux_pwm = components::pwm::PwmMuxComponent::new(&peripherals.pwm)
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
//!             //capsules_extra::servo_motor_pwm::DEFAULT_MAX_BUZZ_TIME_MS,
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
        Ok(())
    }
}
