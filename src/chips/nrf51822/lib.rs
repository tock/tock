#![crate_name = "nrf51822"]
#![crate_type = "rlib"]
#![feature(asm,concat_idents,const_fn)]
#![feature(core_intrinsics)]
#![no_std]

extern crate common;
extern crate hil;

mod peripheral_registers;
mod peripheral_interrupts;
pub mod gpio;

pub mod uart;


