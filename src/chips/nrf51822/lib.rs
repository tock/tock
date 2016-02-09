#![crate_name = "nrf51822"]
#![crate_type = "rlib"]
#![feature(asm,concat_idents,const_fn)]
#![no_std]

extern crate common;
extern crate hil;

pub mod gpio;
