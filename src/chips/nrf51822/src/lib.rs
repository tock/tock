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
pub mod uart;

pub use chip::NRF51822;

#[repr(C)]
pub struct PinCnf(usize);

impl PinCnf {
    pub const unsafe fn new(pin: usize) -> PinCnf {
        PinCnf(pin)
    }
}

