#![feature(asm, concat_idents, const_fn, try_from)]
#![feature(exclusive_range_pattern)]
#![no_std]
#![crate_name = "arty_exx"]
#![crate_type = "rlib"]

extern crate riscv32i;
extern crate sifive;

#[allow(unused_imports)]
#[macro_use(
    debug,
    debug_verbose,
    debug_gpio,
    register_bitfields,
    register_bitmasks
)]
extern crate kernel;

mod interrupts;

pub mod chip;
pub mod gpio;
pub mod uart;
pub mod pwm;
