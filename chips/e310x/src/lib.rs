//! Chip support for the E310 from SiFive.

#![feature(asm, exclusive_range_pattern)]
#![no_std]
#![crate_name = "e310x"]
#![crate_type = "rlib"]

mod interrupts;

pub mod chip;
pub mod gpio;
pub mod plic;
pub mod prci;
pub mod pwm;
pub mod rtc;
pub mod timer;
pub mod uart;
pub mod watchdog;
