#![feature(const_fn, try_from)]
#![feature(in_band_lifetimes)]
#![no_std]

#[allow(unused_imports)]
#[macro_use(
    debug,
    debug_verbose,
    debug_gpio,
    register_bitfields,
    register_bitmasks
)]
extern crate kernel;

pub mod aes;
pub mod constants;
pub mod gpio;
pub mod peripheral_interrupts;
pub mod pinmux;
pub mod rtc;
pub mod temperature;
pub mod timer;
pub mod trng;
