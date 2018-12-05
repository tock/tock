#![feature(const_fn, untagged_unions)]
#![no_std]
#![crate_name = "cc26x2"]
#![crate_type = "rlib"]
extern crate cortexm4;
extern crate tock_rt0;
#[allow(unused_imports)]
#[macro_use]
extern crate kernel;
#[macro_use]
extern crate enum_primitive;

pub mod aon;
pub mod chip;
pub mod crt1;
pub mod gpio;
pub mod i2c;
pub mod peripheral_interrupts;
pub mod prcm;
pub mod rtc;
pub mod trng;
pub mod uart;

pub use crt1::init;
