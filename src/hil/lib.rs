#![crate_name = "hil"]
#![crate_type = "rlib"]
#![feature(asm,lang_items,core,no_std)]
#![no_std]

extern crate core;

pub mod gpio;
pub mod timer;

pub trait Controller {
    type Params;
    type Config;

    fn new(Self::Params) -> Self;
    fn configure(&mut self, Self::Config);
}
