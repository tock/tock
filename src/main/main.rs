#![feature(core,no_std)]
#![no_main]
#![no_std]

extern crate core;
extern crate support;
extern crate hil;
extern crate platform;

use core::prelude::*;

#[no_mangle]
pub extern fn main() {
    use hil::Controller;

    use platform::gpio;
    use hil::gpio::GPIOPin;

    let mut led : gpio::GPIOPin = Controller::new(gpio::Location::GPIOPin10);
    led.configure(None);

    led.enable_output();
    led.toggle();
    let mut i = 0;
    loop {
        i += 1;
        if i == 10000000 {
            led.toggle();
            i = 0;
        }
    }
}

