//! Implementations for generic SiFive MCU peripherals.

#![feature(asm, const_fn, in_band_lifetimes, cell_update)]
#![no_std]
#![crate_name = "sifive"]
#![crate_type = "rlib"]

pub mod gpio;
pub mod prci;
pub mod pwm;
pub mod rtc;
pub mod spi;
pub mod uart;
pub mod watchdog;
