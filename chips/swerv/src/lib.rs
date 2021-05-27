//! Drivers and support modules for SweRV SoCs

#![feature(const_fn_trait_bound)]
#![no_std]
#![crate_name = "swerv"]
#![crate_type = "rlib"]

pub mod eh1_pic;
pub mod eh1_timer;
