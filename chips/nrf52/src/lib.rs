#![feature(asm, concat_idents, const_fn, const_cell_new)]
#![no_std]
#![crate_name = "nrf52"]
#![crate_type = "rlib"]
extern crate cortexm4;
#[allow(unused_imports)]
#[macro_use(debug)]
extern crate kernel;
extern crate nrf5x;

#[macro_use]
extern crate bitfield;

mod peripheral_registers;

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
