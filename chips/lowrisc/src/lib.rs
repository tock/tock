//! Implementations for generic LowRISC peripherals.

#![feature(llvm_asm, const_fn, in_band_lifetimes)]
#![no_std]
#![crate_name = "lowrisc"]
#![crate_type = "rlib"]

pub mod gpio;
pub mod uart;
