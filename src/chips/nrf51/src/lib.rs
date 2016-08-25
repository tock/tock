#![crate_name = "nrf51"]
#![crate_type = "rlib"]
#![feature(asm,concat_idents,const_fn)]
#![feature(core_intrinsics)]
#![no_std]

extern crate common;
extern crate hil;
extern crate main;

extern "C" {
    pub fn init();
}

mod peripheral_registers;
mod peripheral_interrupts;
mod nvic;

pub mod chip;
pub mod gpio;
pub mod rtc;
pub mod timer;
pub mod clock;
pub mod uart;
pub use chip::NRF51;
