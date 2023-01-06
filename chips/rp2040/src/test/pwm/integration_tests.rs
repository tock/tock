//! Integration tests for PWM peripheral
//!
//! This module provides four integration tests:
//!
//! ## hello_pwm
//!
//! This test sets up GPIOs 14 and 15 as PWM pins. GPIO 15 should be much brighter than 14.
//!
//! ## fade_pwm
//!
//! GPIO 12 is set as a PWM pin. Its brightness should increase and decrease over time.
//!
//! ## controllable_pwm
//!
//! GPIO 10 is set as an output PWM pin and GPIO 11 as an PWM input pin. Connect the input pin to a
//! source of 3.3V. The output pin should have a medium brightness. Now, remove the input voltage
//! from the input pin. Depending on the timing, the output LED should either:
//! + stop brighting
//! + bright very strongly
//!
//! The input pin can be connected again to a power supply to start brighting as before.
//!
//! ## synchronious_pwm
//!
//! GPIOs 8 and 6 are set as output PWM pins. They should be both fading, but in opposite
//! directions: when GPIO 6 is at its maximum brightness, GPIO 8 is at its lowest one.
//!
//! ## Running the tests
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
//! let pwm_test = test::pwm::integration_tests::new(peripherals);
//! ```
//!
//! To run a specific test:
//!
//! ```rust,ignore
//! pwm_test.hello_pwm();
//! ```
//!
//! To run all tests at once:
//!
//! ```rust,ignore
//! pwm_test.run_all();
//! ```
//!
//! For more details, see [PwmTest].

use kernel::debug;
use kernel::hil::pwm::PwmPin;
use kernel::hil::pwm::Pwm;
use kernel::utilities::cells::OptionalCell;
use kernel::static_init;

use crate::chip::Rp2040DefaultPeripherals;
use crate::gpio::{RPGpio, GpioFunction};
use crate::pwm;

#[doc(hidden)]
struct PwmInterrupt {
    pwm: &'static pwm::Pwm<'static>,
    fading_channel_number: pwm::ChannelNumber,
    synchronious_channel_1: pwm::ChannelNumber,
    synchronious_channel_2: pwm::ChannelNumber,
    fading_compare_value: OptionalCell<u16>,
    fading_upwards: OptionalCell<bool>,
    synchronious_compare_value: OptionalCell<u16>,
    synchronious_upwards: OptionalCell<bool>
}

#[doc(hidden)]
impl PwmInterrupt {
    fn new(
        pwm: &'static pwm::Pwm,
        fading_gpio: RPGpio,
        synchronious_gpio_1: RPGpio,
        synchronious_gpio_2: RPGpio
    ) -> Self {
        PwmInterrupt {
            pwm,
            fading_channel_number: pwm::ChannelNumber::from(fading_gpio),
            synchronious_channel_1: pwm::ChannelNumber::from(synchronious_gpio_1),
            synchronious_channel_2: pwm::ChannelNumber::from(synchronious_gpio_2),
            fading_compare_value: OptionalCell::new(0),
            synchronious_compare_value: OptionalCell::new(0),
            fading_upwards: OptionalCell::new(true),
            synchronious_upwards: OptionalCell::new(true)
        }
    }
}

#[doc(hidden)]
impl pwm::Interrupt for PwmInterrupt {
    fn fired(&self, channel_number: pwm::ChannelNumber) {
        if channel_number == self.fading_channel_number {
            let mut compare_value = self.fading_compare_value.unwrap_or_panic();
            if self.fading_upwards.unwrap_or_panic() {
                compare_value += 1;
            } else {
                compare_value -= 1;
            }
            self.fading_compare_value.set(compare_value);
            self.pwm.set_compare_values_a_and_b(
                self.fading_channel_number,
                compare_value * compare_value,
                compare_value * compare_value);
            if compare_value == 255 {
                self.fading_upwards.set(false);
            } else if compare_value == 0 {
                self.fading_upwards.set(true);
            }
        }
        else if channel_number == self.synchronious_channel_1 {
            let mut compare_value = self.synchronious_compare_value.unwrap_or_panic();
            if self.synchronious_upwards.unwrap_or_panic() {
                compare_value += 1;
            } else {
                compare_value -= 1;
            }
            self.synchronious_compare_value.set(compare_value);
            self.pwm.set_compare_values_a_and_b(
                self.synchronious_channel_1,
                compare_value * compare_value,
                compare_value * compare_value);
            self.pwm.set_compare_values_a_and_b(
                self.synchronious_channel_2,
                compare_value * compare_value,
                compare_value * compare_value);
            if compare_value == 255 {
                self.synchronious_upwards.set(false);
            } else if compare_value == 0 {
                self.synchronious_upwards.set(true);
            }
        }
    }
}

/// Struct used to run integration tests
pub struct PwmTest {
    peripherals: &'static Rp2040DefaultPeripherals<'static>
}

/// Create a PwmTest to run tests
pub fn new(peripherals: &'static Rp2040DefaultPeripherals<'static>) -> PwmTest {
    let pwm_interrupt = unsafe {
        static_init!(PwmInterrupt, PwmInterrupt::new(
                &peripherals.pwm,
                RPGpio::GPIO12,
                RPGpio::GPIO8,
                RPGpio::GPIO6
        ))
    };
    peripherals.pwm.set_interrupt_handler(pwm_interrupt);
    PwmTest { peripherals }
}

impl PwmTest {

    /// Run hello_pwm test
    pub fn hello_pwm(&self) {
        self.peripherals.pins.get_pin(RPGpio::GPIO14).set_function(GpioFunction::PWM);
        self.peripherals.pins.get_pin(RPGpio::GPIO15).set_function(GpioFunction::PWM);
        let pwm_pin_14 = self.peripherals.pwm.gpio_to_pwm_pin(RPGpio::GPIO14);
        let max_freq = pwm_pin_14.get_maximum_frequency_hz();
        let max_duty_cycle = pwm_pin_14.get_maximum_duty_cycle();
        assert_eq!(pwm_pin_14.start(max_freq / 8, max_duty_cycle / 2), Ok(()));
        let pwm = &self.peripherals.pwm;
        debug!("PWM pin 14 started");
        let max_freq = pwm.get_maximum_frequency_hz();
        let max_duty_cycle = pwm.get_maximum_duty_cycle();
        assert_eq!(pwm.start(&RPGpio::GPIO15, max_freq / 8, max_duty_cycle / 8 * 7), Ok(()));
        debug!("PWM pin 15 started");
    }

    /// Run fading_pwm
    pub fn fading_pwm(&self) {
        self.peripherals.pins.get_pin(RPGpio::GPIO12).set_function(GpioFunction::PWM);
        let pwm = &self.peripherals.pwm;
        let channel_number = pwm.gpio_to_pwm_pin(RPGpio::GPIO12).get_channel_number();
        pwm.enable_interrupt(channel_number);
        pwm.set_divider_int_frac(channel_number, 8, 0);
        pwm.set_enabled(channel_number, true);
        debug!("PWM pin 12 started");
    }

    /// Run controllable_pwm test
    pub fn controllable_pwm(&self) {
        self.peripherals.pins.get_pin(RPGpio::GPIO10).set_function(GpioFunction::PWM);
        self.peripherals.pins.get_pin(RPGpio::GPIO11).set_function(GpioFunction::PWM);
        let pwm = &self.peripherals.pwm;
        let channel_number = pwm.gpio_to_pwm_pin(RPGpio::GPIO10).get_channel_number();
        pwm.set_compare_value_a(channel_number, 26214); //40% duty cycle
        pwm.set_div_mode(channel_number, pwm::DivMode::High);
        pwm.set_enabled(channel_number, true);
        debug!("PWM pin 10 started");
    }

    /// Run synchronious_pwm test
    pub fn synchronious_pwm(&self) {
        self.peripherals.pins.get_pin(RPGpio::GPIO8).set_function(GpioFunction::PWM);
        self.peripherals.pins.get_pin(RPGpio::GPIO6).set_function(GpioFunction::PWM);
        let pwm = &self.peripherals.pwm;
        let mut config: pwm::PwmChannelConfiguration = Default::default();
        config.set_divider_int_frac(32, 0);
        let mut mask = 0u8;
        let channel_number = pwm.gpio_to_pwm_pin(RPGpio::GPIO8).get_channel_number();
        pwm.configure_channel(channel_number, &config);
        mask |= 1 << channel_number as u8;
        pwm.enable_interrupt(channel_number);
        let channel_number = pwm.gpio_to_pwm_pin(RPGpio::GPIO6).get_channel_number();
        mask |= 1 << channel_number as u8;
        pwm.configure_channel(channel_number, &config);
        pwm.set_invert_polarity(channel_number, true, false);
        pwm.set_mask_enabled(mask);
        debug!("PWM pin 8 and 6 started");
    }

    /// Run all integration tests
    pub fn run_all(&self) {
        self.hello_pwm();
        self.fading_pwm();
        self.controllable_pwm();
        self.synchronious_pwm();
    }
}
