//! Drivers and chip support for the E21 soft core.

#![feature(asm)]
#![no_std]
#![crate_name = "arty_e21"]
#![crate_type = "rlib"]

mod interrupts;

pub mod chip;
pub mod gpio;
pub mod timer;
pub mod uart;
