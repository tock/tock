//! Implementations for generic SiFive MCU peripherals.

#![feature(const_fn_trait_bound)]
#![no_std]
#![crate_name = "sifive"]
#![crate_type = "rlib"]

pub mod clint;
pub mod gpio;
pub mod prci;
pub mod pwm;
pub mod rtc;
pub mod uart;
pub mod watchdog;
