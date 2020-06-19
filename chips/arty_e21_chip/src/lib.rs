//! Drivers and chip support for the E21 soft core.

#![feature(llvm_asm)]
#![no_std]
#![crate_name = "arty_e21_chip"]
#![crate_type = "rlib"]

mod interrupts;

pub mod chip;
pub mod gpio;
pub mod timer;
pub mod uart;
