#![crate_name = "process"]
#![crate_type = "rlib"]
#![no_std]
#![feature(core_intrinsics,raw,core_slice_ext,no_std)]

extern crate common;

pub mod process;
pub mod callback;

pub use process::{Process,State};
pub use callback::Callback;

