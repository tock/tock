#![no_std]
#![crate_name = "cc26x2"]
#![crate_type = "rlib"]
extern crate cc26xx;
extern crate cortexm4;
#[allow(unused_imports)]
#[macro_use(debug)]
extern crate kernel;

pub mod chip;
pub mod crt1;

pub use crt1::init;
