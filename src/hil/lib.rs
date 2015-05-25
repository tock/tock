#![crate_name = "hil"]
#![crate_type = "rlib"]
#![feature(asm,lang_items,core,no_std)]
#![no_std]

extern crate core;

pub mod gpio;
pub mod timer;
pub mod uart;

pub trait Controller {
    type Config;

    fn configure(&mut self, Self::Config);
}
