#![crate_name = "hil"]
#![crate_type = "rlib"]
#![feature(asm,lang_items,core,no_std)]
#![no_std]

extern crate core;
extern crate process;

pub mod gpio;
pub mod i2c;
pub mod timer;
pub mod uart;
pub mod adc;

pub use process::{Callback, AppPtr};

pub trait Controller {
    type Config;

    fn configure(&mut self, Self::Config);
}

pub trait Driver {
    fn subscribe(&mut self, subscribe_type: usize, callback: Callback) -> isize;
    fn command(&mut self, cmd_type: usize, r2: usize) -> isize;

    #[allow(unused)]
    fn allow(&mut self, allow_type: usize, slice: AppPtr<u8>) -> isize {
        -1
    }
}

