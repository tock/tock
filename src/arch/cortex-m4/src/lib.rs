#![crate_name = "cortexm4"]
#![crate_type = "rlib"]
#![feature(const_fn)]
#![no_std]

extern crate common;
extern crate main;

pub mod mpu;
pub mod systick;

