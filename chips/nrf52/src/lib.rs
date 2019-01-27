#![feature(asm, const_fn, core_intrinsics, try_from)]
#![no_std]
#![crate_name = "nrf52"]
#![crate_type = "rlib"]

pub mod adc;
pub mod chip;
pub mod clock;
pub mod crt1;
mod deferred_call_tasks;
pub mod deferred_call_mux;
pub mod ficr;
pub mod i2c;
pub mod nvmc;
pub mod ppi;
pub mod pwm;
pub mod radio;
pub mod spi;
pub mod uart;
pub mod uicr;

pub use crate::crt1::init;
