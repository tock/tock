//! Drivers and chip support for ESP32-C3.

#![feature(const_fn_trait_bound, naked_functions, asm)]
#![no_std]
#![crate_name = "esp32_c3"]
#![crate_type = "rlib"]

pub mod chip;
pub mod intc;
pub mod interrupts;
