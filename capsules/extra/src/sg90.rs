// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

use core::cell::Cell;
use core::mem::size_of;
use kernel::hil;
use kernel::ErrorCode;
pub struct Sg90<'a, P: hil::pwm::PwmPin> {
    /// The underlying PWM generator to change the angle.
    pwm_pin: &'a P,
    /// Stores the angle everytime it changes.
    current_angle: Cell<Option<usize>>,
}

impl<'a, P: hil::pwm::PwmPin> Sg90<'a, P> {
    pub fn new(pwm_pin: &'a P) -> Sg90<'a, P> {
        Sg90 {
            pwm_pin,
            current_angle: Cell::new(None),
        }
    }
}

impl<'a, P: hil::pwm::PwmPin> kernel::hil::servo::Servo<'a> for Sg90<'a, P> {
    fn set_angle(&self, angle: u16) -> Result<(), ErrorCode> {
        // The assert! macro ensures that the code will not compile on platforms
        // where `usize` is smaller than `u16`.
        const _: () = assert!(size_of::<usize>() >= size_of::<u16>());
        if angle <= 180 {
            self.current_angle.set(Some(angle as usize));
            // As specified in the datasheet:
            // https://www.friendlywire.com/projects/ne555-servo-safe/SG90-datasheet.pdf,
            // the frequency used for sg90 servo is always 50hz.
            const FREQUENCY_HZ: usize = 50;
            // This calculates the pulse width in microseconds for a specific angle.
            // 500 and 2000 miliseconds define the range within
            // which the angle can be set to any position.
            let pulse_width_us = 500 + 2000 / 180 * (angle as usize);
            // The duty_cycle formula is (pulse_width/period)*100.
            // The period is 20 000 miliseconds (also specified in the datasheet).
            // If we simplify we're left with pulse_width/20.
            // We also need to scale this to the maximum duty_cycle suported by the pin.
            // We do this by multiplying the value we get from the
            // get_maximum_duty_cycle() function with pulse_width/20 and divide it by 100.
            // This leaves us with the below formula:
            let duty_cycle = pulse_width_us * self.pwm_pin.get_maximum_duty_cycle() / 20000;
            self.pwm_pin.start(FREQUENCY_HZ, duty_cycle)?;
            Ok(())
        } else {
            Err(ErrorCode::INVAL)
        }
    }

    fn get_angle(&self) -> Result<usize, ErrorCode> {
        //The SG90 servomotor cannot return its angle.
        Err(ErrorCode::NOSUPPORT)
    }
}
