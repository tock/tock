//! A library for common operations in the Tock OS.

#![crate_name = "common"]
#![crate_type = "rlib"]
#![feature(core,no_std)]
#![no_std]

extern crate core;
extern crate support;
extern crate hil;

pub mod shared;
pub mod ring_buffer;
pub mod led;
