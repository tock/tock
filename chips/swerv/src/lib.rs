//! Drivers and support modules for SweRV SoCs

#![feature(const_fn, asm)]
#![no_std]
#![crate_name = "swerv"]
#![crate_type = "rlib"]

pub mod eh1_pic;
