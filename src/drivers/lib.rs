#![crate_name = "drivers"]
#![crate_type = "rlib"]
#![feature(core,core_str_ext,core_prelude,core_slice_ext,no_std)]
#![no_std]

extern crate core;
extern crate hil;

mod std {
   pub use core::*; 
}

pub mod gpio;
pub mod console;
pub mod tmp006;
