//! Tests for PWM peripheral

use kernel::debug;
use kernel::hil::pwm::PwmPin;

use rp2040::chip::Rp2040DefaultPeripherals;
use rp2040::gpio::{RPGpio, GpioFunction};

pub struct PwmTest {
    peripherals: &'static Rp2040DefaultPeripherals<'static>
}

impl PwmTest {
    pub fn new(peripherals: &'static Rp2040DefaultPeripherals<'static>) -> PwmTest {
        PwmTest { peripherals }
    }

    pub fn hello_world_hil(&self) {
        self.peripherals.pins.get_pin(RPGpio::GPIO2).set_function(GpioFunction::PWM);
        self.peripherals.pins.get_pin(RPGpio::GPIO3).set_function(GpioFunction::PWM);
        let pwm_pin_2 = self.peripherals.pwm.gpio_to_pwm_pin(RPGpio::GPIO2);
        let max_freq = pwm_pin_2.get_maximum_frequency_hz();
        let max_duty_cycle = pwm_pin_2.get_maximum_duty_cycle();
        assert_eq!(pwm_pin_2.start(max_freq / 8, max_duty_cycle / 8 * 5), Ok(()));
        debug!("PWM pin 2 started");
    }
}
