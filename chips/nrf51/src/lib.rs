#![feature(asm, concat_idents, const_fn, const_cell_new)]
#![no_std]
#![crate_name = "nrf51"]
#![crate_type = "rlib"]
extern crate cortexm0;
#[allow(unused_imports)]
#[macro_use(debug)]
extern crate kernel;
extern crate nrf5x;

mod peripheral_registers;

pub mod chip;
pub mod crt1;
pub mod uart;
pub mod radio;

pub use crt1::init;
