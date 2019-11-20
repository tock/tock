#![no_std]

pub use nrf52::{
    adc, aes, ble_radio, chip, clock, ficr, i2c, init, nvmc, pinmux, pwm, rtc, temperature, trng,
    uart, uicr,
};
pub mod gpio;
