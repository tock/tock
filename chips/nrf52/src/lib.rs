#![feature(asm, concat_idents, const_fn, const_cell_new, try_from)]
#![no_std]
#![crate_name = "nrf52"]
#![crate_type = "rlib"]
#[warn(missing_docs)]
#[allow(unused_imports)]
extern crate cortexm4;
extern crate nrf5x;

#[macro_use]
extern crate bitfield;
#[macro_use(debug, register_bitfields, register_bitmasks)]
extern crate kernel;

mod peripheral_registers;

pub mod clock;
pub mod chip;
pub mod crt1;
pub mod ficr;
pub mod nvmc;
pub mod radio;
pub mod uart;
pub mod uicr;
pub mod spi;
pub mod i2c;

pub use crt1::init;
