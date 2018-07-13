#![feature(const_fn, used)]
#![no_std]
#![crate_name = "cc26xx"]
#![crate_type = "rlib"]
#[allow(unused_imports)]
#[macro_use]
extern crate kernel;

pub mod aon;
pub mod gpio;
pub mod ioc;
pub mod peripheral_interrupts;
pub mod prcm;
pub mod rtc;
pub mod trng;
pub mod uart;
