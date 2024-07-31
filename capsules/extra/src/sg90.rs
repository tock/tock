// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

use kernel::hil;
use kernel::ErrorCode;
pub struct Sg90<'a, P: hil::pwm::PwmPin> {
    /// The underlying PWM generator to change the angle.
    pwm_pin: &'a P,
}

impl<'a, P: hil::pwm::PwmPin> Sg90<'a, P> {
    pub fn new(pwm_pin: &'a P) -> Sg90<'a, P> {
        Sg90 { pwm_pin: pwm_pin }
    }
}

impl<'a, P: hil::pwm::PwmPin> kernel::hil::servo::Servo<'a> for Sg90<'a, P> {
    fn servo(&self, angle: usize) -> Result<(), ErrorCode> {
        if angle <= 180 {
            // The frequency used for sg90 servo is always 50hz.
            let frequency_hz = 50;
            // This calculates the pulse width in microseconds for a specific angle.
            // We substract 50 from angle to adapt how the servo uses angles to the
            // trigonometric system.
            let pulse_width_us = 1000 + 1000 / 90 * (angle - 50);
            // The duty_cycle formula is (pulse_width/period)*100.
            // The period is usually 20 000 miliseconds.
            // If we simplify we're left with pulse_width/20.
            // We also need to scale this to the maximum duty_cycle suported by the pin.
            // We do this by multiplying the value we get from the
            // get_maximum_duty_cycle() function with pulse_width/20 and divide it by 100.
            // This leaves us with the below formula:
            let duty_cycle = pulse_width_us * self.pwm_pin.get_maximum_duty_cycle() / 20000;
            self.pwm_pin.start(frequency_hz, duty_cycle)?;
            Ok(())
        } else {
            Err(ErrorCode::INVAL)
        }
    }
}
