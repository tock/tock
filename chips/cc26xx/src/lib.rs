#![feature(asm, concat_idents, const_fn, const_cell_new, try_from)]
#![no_std]
#![crate_name = "cc26xx"]
#![crate_type = "rlib"]
#[allow(unused_imports)]
#[macro_use]
extern crate kernel;

pub mod aon;
pub mod rtc;
pub mod gpio;
pub mod ioc;
pub mod prcm;
pub mod ccfg;
pub mod trng;
pub mod peripheral_interrupts;
