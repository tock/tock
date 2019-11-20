#![feature(asm, const_fn, core_intrinsics)]
#![no_std]
#![crate_name = "nrf52"]
#![crate_type = "rlib"]

pub mod adc;
pub mod ble_radio;
pub mod chip;
pub mod clock;
pub mod crt1;
mod deferred_call_tasks;
pub mod ficr;
pub mod i2c;
pub mod ieee802154_radio;
pub mod nvmc;
pub mod ppi;
pub mod pwm;
pub mod spi;
pub mod uart;
pub mod uicr;

pub use crate::crt1::init;
pub use nrf5x::{aes, gpio, pinmux, rtc, temperature, trng};
