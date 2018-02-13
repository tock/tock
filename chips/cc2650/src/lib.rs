#![feature(asm, concat_idents, const_fn, const_cell_new, try_from)]
#![no_std]
#![crate_name = "cc2650"]
#![crate_type = "rlib"]
extern crate cortexm3;
#[allow(unused_imports)]
#[macro_use(debug)]
extern crate kernel;

extern crate bitfield;

pub mod aon;
pub mod chip;
pub mod crt1;
pub mod gpio;
pub mod prcm;
pub mod ccfg;
pub mod peripheral_interrupts;

mod ioc;

pub use crt1::init;
