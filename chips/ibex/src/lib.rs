//! Drivers and chip support for the Ibex soft core.

#![feature(asm, concat_idents, const_fn, naked_functions)]
#![feature(in_band_lifetimes)]
#![no_std]
#![crate_name = "ibex"]
#![crate_type = "rlib"]

mod interrupts;

pub mod aes;
pub mod chip;
pub mod gpio;
pub mod plic;
pub mod timer;
pub mod uart;
