//! Drivers and chip support for the Ibex soft core.

#![feature(asm, concat_idents, const_fn)]
#![feature(exclusive_range_pattern)]
#![no_std]
#![crate_name = "ibex"]
#![crate_type = "rlib"]

mod interrupts;

pub mod chip;
pub mod plic;
pub mod uart;
