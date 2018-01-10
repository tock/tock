#![allow(unused)]

use mk20::{gpio, clock};
use mk20::gpio::*;

fn delay() {
    unsafe {
        for _ in 1..2000_000 {
            asm!("nop" :::: "volatile");
        }
    }
}

pub fn blink_test() {
    loop {
        led_toggle();
        delay();
    }
}

pub fn led_on() {
    unsafe {
        PC05.release_claim();
        let led = PC05.claim_as_gpio();
        led.enable_output();
        led.set();
    }
}

pub fn led_off() {
    unsafe {
        PC05.release_claim();
        let led = PC05.claim_as_gpio();
        led.enable_output();
        led.clear();
    }
}

pub fn led_toggle() {
    unsafe {
        PC05.release_claim();
        let led = PC05.claim_as_gpio();
        led.enable_output();
        led.toggle();
    }
}
