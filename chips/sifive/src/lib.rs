#![feature(asm, concat_idents, const_fn, core_intrinsics)]
#![feature(in_band_lifetimes)]
#![feature(exclusive_range_pattern)]
#![no_std]
#![crate_name = "sifive"]
#![crate_type = "rlib"]

pub mod gpio;
pub mod prci;
pub mod pwm;
pub mod rtc;
pub mod uart;
pub mod watchdog;
