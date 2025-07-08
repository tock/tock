// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Integration tests for PWM peripheral
//!
//! This module provides four integration tests:
//!
//! ## hello_pwm
//!
//! This test sets up GPIOs 14 and 15 as PWM pins. GPIO 15 should be much brighter than 14.
//!
//! ## Running the test
//!
//! First step is including the test module:
//!
//! ```rust,ignore
//! #[allow(dead_code)]
//! use rp2040::test;
//! ```
//!
//! Then create a test instance:
//!
//! ```rust,ignore
//! let pwm_test = test::pwm::new(peripherals);
//! ```
//!
//! Then run the test:
//!
//! ```rust,ignore
//! pwm_test.hello_pwm();
//! ```

use kernel::debug;
use kernel::hil::pwm::Pwm;
use kernel::hil::pwm::PwmPin;

use crate::chip::Rp2040DefaultPeripherals;
use crate::gpio::{GpioFunction, RPGpio};

/// Struct used to run integration tests
pub struct PwmTest {
    peripherals: &'static Rp2040DefaultPeripherals<'static>,
}

/// Create a PwmTest to run tests
pub fn new(peripherals: &'static Rp2040DefaultPeripherals<'static>) -> PwmTest {
    PwmTest { peripherals }
}

impl PwmTest {
    /// Run hello_pwm test
    pub fn hello_pwm(&self) {
        self.peripherals
            .pins
            .get_pin(RPGpio::GPIO14)
            .set_function(GpioFunction::PWM);
        self.peripherals
            .pins
            .get_pin(RPGpio::GPIO15)
            .set_function(GpioFunction::PWM);
        let pwm_pin_14 = self.peripherals.pwm.gpio_to_pwm_pin(RPGpio::GPIO14);
        let max_freq = pwm_pin_14.get_maximum_frequency_hz();
        let max_duty_cycle = pwm_pin_14.get_maximum_duty_cycle();
        assert_eq!(pwm_pin_14.start(max_freq / 8, max_duty_cycle / 2), Ok(()));
        let pwm = &self.peripherals.pwm;
        debug!("PWM pin 14 started");
        let max_freq = pwm.get_maximum_frequency_hz();
        let max_duty_cycle = pwm.get_maximum_duty_cycle();
        assert_eq!(
            pwm.start(&RPGpio::GPIO15, max_freq / 8, max_duty_cycle / 8 * 7),
            Ok(())
        );
        debug!("PWM pin 15 started");
    }
}
