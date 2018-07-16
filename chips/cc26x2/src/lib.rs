#![feature(const_fn, used)]
#![no_std]
#![crate_name = "cc26x2"]
#![crate_type = "rlib"]
extern crate cc26xx;
extern crate cortexm4;
#[allow(unused_imports)]
#[macro_use]
extern crate kernel;

pub mod aon;
pub mod chip;
pub mod crt1;
pub mod i2c;
pub mod prcm;
pub mod rtc;

pub use crt1::init;
