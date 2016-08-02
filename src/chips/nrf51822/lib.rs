#![crate_name = "nrf51822"]
#![crate_type = "rlib"]
#![feature(asm,concat_idents,const_fn,core_intrinsics)]
#![no_std]

extern crate common;
extern crate hil;
extern crate main;

extern {
    pub fn init();
}

mod peripheral_registers;
pub mod gpio;
mod chip;

pub use chip::NRF51822;

