#![no_std]

pub use nrf52::{
    adc, aes, ble_radio, clock, constants, crt1, ficr, i2c, ieee802154_radio, init, nvmc,
    peripheral_interrupts, pinmux, ppi, pwm, rtc, spi, temperature, timer, trng, uart, uicr,
};
pub mod chip;
pub mod gpio;
pub mod interrupt_service;
