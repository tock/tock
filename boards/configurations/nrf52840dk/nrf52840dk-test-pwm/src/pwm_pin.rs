// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! A `hil::pwm::PwmPin` for a `hil::pwm::Pwm` that is dedicated to one pin.
//!
//! This looks a lot like
//! `capsules_core::virtualizers::virtual_pwm::PwmPinUser`, except it does not
//! multiplex several pins onto a shared `MuxPwm`. It assumes the underlying
//! `Pwm` instance is exclusively owned by this one pin.

use kernel::ErrorCode;
use kernel::hil;

/// Exposes a single pin on a dedicated (non-shared) PWM peripheral as a
/// `PwmPin`.
pub struct PwmPinStatic<'a, P: hil::pwm::Pwm> {
    pwm: &'a P,
    pin: P::Pin,
}

impl<'a, P: hil::pwm::Pwm> PwmPinStatic<'a, P> {
    pub const fn new(pwm: &'a P, pin: P::Pin) -> Self {
        PwmPinStatic { pwm, pin }
    }
}

impl<P: hil::pwm::Pwm> hil::pwm::PwmPin for PwmPinStatic<'_, P> {
    fn start(&self, frequency_hz: usize, duty_cycle: usize) -> Result<(), ErrorCode> {
        self.pwm.start(&self.pin, frequency_hz, duty_cycle)
    }

    fn stop(&self) -> Result<(), ErrorCode> {
        self.pwm.stop(&self.pin)
    }

    fn get_maximum_frequency_hz(&self) -> usize {
        self.pwm.get_maximum_frequency_hz()
    }

    fn get_maximum_duty_cycle(&self) -> usize {
        self.pwm.get_maximum_duty_cycle()
    }
}
