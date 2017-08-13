#![feature(asm,concat_idents,const_fn)]
#![no_std]

#[allow(unused_imports)]
#[macro_use(debug)]
extern crate kernel;

mod peripheral_registers;

pub mod aes;
pub mod clock;
pub mod gpio;
pub mod nvic;
pub mod peripheral_interrupts;
pub mod pinmux;
pub mod rtc;
pub mod timer;
pub mod temperature;
pub mod trng;
