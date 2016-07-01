#![crate_name = "hil"]
#![crate_type = "rlib"]
#![feature(asm,lang_items,const_fn)]
#![no_std]

extern crate common;
extern crate process;

pub mod driver;

pub mod led;
pub mod alarm;
pub mod gpio;
pub mod i2c;
pub mod spi_master;
pub mod timer;
pub mod uart;
pub mod adc;
pub mod flash;

pub use driver::Driver;

pub trait Controller {
    type Config;

    fn configure(&self, Self::Config);
}

