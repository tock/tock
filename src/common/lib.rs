//! A library for common operations in the Tock OS.

#![crate_name = "common"]
#![crate_type = "rlib"]
#![feature(core,core_slice_ext,core_prelude,no_std)]
#![no_std]

extern crate core;
extern crate support;

pub mod shared;
pub mod ring_buffer;

