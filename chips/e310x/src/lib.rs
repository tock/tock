//! Chip support for the E310 from SiFive.

#![no_std]
#![crate_name = "e310x"]
#![crate_type = "rlib"]

pub mod chip;
pub mod clint;
pub mod gpio;
pub mod plic;
pub mod prci;
pub mod pwm;
pub mod rtc;
pub mod uart;
pub mod watchdog;
