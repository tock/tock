#![feature(asm, concat_idents, const_fn, const_cell_new, try_from, core_intrinsics)]
#![no_std]
#![crate_name = "nrf52"]
#![crate_type = "rlib"]

#[allow(unused_imports)]
extern crate cortexm4;
extern crate nrf5x;

#[allow(unused)]
#[macro_use(debug, debug_verbose, debug_gpio, register_bitfields, register_bitmasks)]
extern crate kernel;

pub mod chip;
pub mod clock;
pub mod crt1;
pub mod ficr;
pub mod i2c;
pub mod nvmc;
pub mod ppi;
pub mod radio;
pub mod spi;
pub mod uart;
pub mod uicr;

pub use crt1::init;
