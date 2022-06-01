//! Drivers and chip support for SweRVolf.

#![feature(naked_functions)]
#![no_std]
#![crate_name = "swervolf_eh1"]
#![crate_type = "rlib"]

pub mod chip;
pub mod syscon;
pub mod uart;
