#![crate_name = "nrf51822"]
#![crate_type = "rlib"]
#![feature(asm,core_intrinsics,concat_idents,const_fn)]
#![no_std]

extern crate hil;
pub mod gpio;
