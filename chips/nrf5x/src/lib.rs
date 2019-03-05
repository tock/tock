#![feature(const_fn, in_band_lifetimes)]
#![no_std]

pub mod aes;
pub mod constants;
pub mod gpio;
pub mod peripheral_interrupts;
pub mod pinmux;
pub mod rtc;
pub mod temperature;
pub mod timer;
pub mod trng;
