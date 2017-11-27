#![feature(asm,concat_idents,const_fn,const_cell_new)]
#![no_std]

#[allow(unused_imports)]
#[macro_use(debug)]
extern crate kernel;

mod peripheral_registers;

pub mod aes;
pub mod ble_advertising_driver;
pub mod ble_advertising_hil;
pub mod clock;
pub mod gpio;
pub mod peripheral_interrupts;
pub mod pinmux;
pub mod rtc;
pub mod timer;
pub mod temperature;
pub mod trng;
