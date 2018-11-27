#![feature(asm, concat_idents, const_fn, try_from)]
#![feature(exclusive_range_pattern)]
#![no_std]
#![crate_name = "sifive"]
#![crate_type = "rlib"]

extern crate riscv32i;

#[allow(unused_imports)]
#[macro_use(
    debug,
    debug_verbose,
    debug_gpio,
    register_bitfields,
    register_bitmasks
)]
extern crate kernel;

pub mod gpio;
pub mod uart;
pub mod watchdog;
pub mod rtc;
pub mod pwm;
pub mod prci;
