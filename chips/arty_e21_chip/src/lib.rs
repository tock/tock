//! Drivers and chip support for the E21 soft core.

#![no_std]
#![crate_name = "arty_e21_chip"]
#![crate_type = "rlib"]

mod interrupts;

pub mod chip;
pub mod clint;
pub mod gpio;
pub mod uart;
