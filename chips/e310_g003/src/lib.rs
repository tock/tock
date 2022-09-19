//! Chip support for the E310-G003 from SiFive.

#![no_std]
#![crate_name = "e310_g003"]
#![crate_type = "rlib"]

pub use e310x::{chip, clint, gpio, plic, prci, pwm, rtc, uart, watchdog};

pub mod interrupt_service;
mod interrupts;
