#![feature(const_fn, untagged_unions)]
#![no_std]
#![crate_name = "cc26x2"]
#![crate_type = "rlib"]

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

pub use crate::crt1::init;
