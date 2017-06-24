#![feature(asm,concat_idents,const_fn)]
#![no_std]

#[allow(unused_imports)]
#[macro_use(debug)]
extern crate kernel;

extern "C" {
    pub fn init();
}

mod peripheral_interrupts;
mod nvic;


pub mod chip;
pub use chip::NRF52;
