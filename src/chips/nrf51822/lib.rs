#![crate_name = "nrf51822"]
#![crate_type = "rlib"]
#![feature(asm,concat_idents,const_fn)]
#![feature(core_intrinsics)]
#![no_std]

extern crate common;
extern crate hil;

mod peripheral_registers;
mod peripheral_interrupts;
mod nvic;
<<<<<<< HEAD
mod helpers;
pub mod chip;
pub mod gpio;
pub mod rtc;
pub mod uart;
=======

pub mod chip;
pub mod gpio;
pub mod rtc;
pub mod timer;
>>>>>>> 9dcf92d1c00ef8fd4fa422cfe04d12850b7da8cf
