#![feature(asm,concat_idents,const_fn)]
#![no_std]

#[allow(unused_imports)]
#[macro_use(debug)]
extern crate kernel;

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
pub mod pinmux;
pub use chip::NRF51;
<<<<<<< HEAD
pub mod temperature;
=======
>>>>>>> 8c1aa42453243d9d528d8489598e7195afd5177e
pub mod trng;
