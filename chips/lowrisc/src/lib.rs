//! Implementations for generic LowRISC peripherals.

#![feature(asm, concat_idents, const_fn, core_intrinsics)]
#![feature(in_band_lifetimes)]
#![feature(exclusive_range_pattern)]
#![no_std]
#![crate_name = "lowrisc"]
#![crate_type = "rlib"]

pub mod uart;
