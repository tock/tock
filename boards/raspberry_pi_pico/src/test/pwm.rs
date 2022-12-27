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

    pub fn hello_pwm(&self) {
        self.peripherals.pins.get_pin(RPGpio::GPIO14).set_function(GpioFunction::PWM);
        self.peripherals.pins.get_pin(RPGpio::GPIO15).set_function(GpioFunction::PWM);
        let pwm_pin_14 = self.peripherals.pwm.gpio_to_pwm_pin(RPGpio::GPIO14);
        let max_freq = pwm_pin_14.get_maximum_frequency_hz();
        let max_duty_cycle = pwm_pin_14.get_maximum_duty_cycle();
        assert_eq!(pwm_pin_14.start(max_freq / 8, max_duty_cycle / 8 * 5), Ok(()));
        debug!("PWM pin 14 started");
        let pwm_pin_15 = self.peripherals.pwm.gpio_to_pwm_pin(RPGpio::GPIO15);
        let max_freq = pwm_pin_15.get_maximum_frequency_hz();
        let max_duty_cycle = pwm_pin_15.get_maximum_duty_cycle();
        assert_eq!(pwm_pin_15.start(max_freq / 8, max_duty_cycle / 8 * 7), Ok(()));
        debug!("PWM pin 15 started");
    }

    pub fn fading_pwm(&self) {
        self.peripherals.pins.get_pin(RPGpio::GPIO12).set_function(GpioFunction::PWM);
        let pwm = &self.peripherals.pwm;
        let channel_number = pwm.gpio_to_pwm_pin(RPGpio::GPIO12).get_channel_number();
        pwm.enable_interrupt(channel_number);
        pwm.set_divider_int_frac(channel_number, 2, 0);
        pwm.set_enabled(channel_number, true);
        debug!("PWM pin 12 started");
        let mut compare_value = 0isize;
        let mut dif = 1isize;
        loop {
            compare_value += dif;
            while pwm.get_counter(channel_number) != 0 {}
            pwm.set_compare_value_a(channel_number, (compare_value * compare_value).try_into().unwrap());
            if compare_value == 255 {
                dif = -dif;
            }
            else if compare_value == 0 {
                dif = -dif;
            }
        }
    }

    pub fn synchronious_start(&self) {
        self.peripherals.pins.get_pin(RPGpio::GPIO6).set_function(GpioFunction::PWM);
        self.peripherals.pins.get_pin(RPGpio::GPIO8).set_function(GpioFunction::PWM);
        self.peripherals.pins.get_pin(RPGpio::GPIO10).set_function(GpioFunction::PWM);
        let pwm = &self.peripherals.pwm;
        let mut mask = 0u8;
        let channel_number = pwm.gpio_to_pwm_pin(RPGpio::GPIO6).get_channel_number();
        mask |= 1 << channel_number as u8;
        pwm.set_compare_value_a(channel_number, 32768); // 50% duty cycle
        let channel_number = pwm.gpio_to_pwm_pin(RPGpio::GPIO8).get_channel_number();
        mask |= 1 << channel_number as u8;
        pwm.set_compare_value_a(channel_number, 32768); // 50% duty cycle
        let channel_number = pwm.gpio_to_pwm_pin(RPGpio::GPIO10).get_channel_number();
        mask |= 1 << channel_number as u8;
        pwm.set_compare_value_a(channel_number, 32768); // 50% duty cycle
        pwm.set_mask_enabled(mask);
    }
}
