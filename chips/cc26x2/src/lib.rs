#![feature(const_fn, untagged_unions, used)]
#![no_std]
#![crate_name = "cc26x2"]
#![crate_type = "rlib"]
extern crate cortexm4;
#[allow(unused_imports)]
#[macro_use]
extern crate kernel;
#[macro_use]
extern crate bitfield;
extern crate fixedvec;

pub mod aon;
pub mod aux;
pub mod chip;
pub mod commands;
pub mod crt1;
pub mod osc;
pub mod oscfh;
pub mod gpio;
pub mod i2c;
pub mod ioc;
pub mod peripheral_interrupts;
pub mod prcm;
pub mod rat;
pub mod rfc;
pub mod rtc;
pub mod trng;
pub mod uart;

pub use crt1::init;
