#![feature(asm, concat_idents, const_fn, const_cell_new, try_from, used)]
#![no_std]
#![crate_name = "nrf51"]
#![crate_type = "rlib"]

extern crate cortexm0;
extern crate nrf5x;

#[allow(unused_imports)]
#[macro_use(debug, debug_verbose, debug_gpio, register_bitfields, register_bitmasks)]
extern crate kernel;

pub mod chip;
pub mod clock;
pub mod crt1;
pub mod i2c;
pub mod radio;
pub mod uart;

pub use crt1::init;
