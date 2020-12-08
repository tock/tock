#![no_std]

pub use nrf52::{
    acomp, adc, aes, ble_radio, chip, clock, constants, crt1, deferred_call_tasks, ficr, i2c,
    ieee802154_radio, init, nvmc, peripheral_interrupts as base_interrupts, pinmux, power, ppi,
    pwm, rtc, spi, temperature, timer, trng, uart, uicr, usbd,
};
pub mod gpio;
pub mod interrupt_service;

pub mod peripheral_interrupts;
