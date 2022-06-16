//! Drivers and chip support for ESP32-C3.

#![feature(naked_functions)]
#![no_std]
#![crate_name = "esp32_c3"]
#![crate_type = "rlib"]

pub mod chip;
pub mod intc;
pub mod interrupts;
pub mod sysreg;

pub mod timg {
    pub use esp32::timg::{ClockSource, TIMG0_BASE, TIMG1_BASE};
    pub type TimG<'a> = esp32::timg::TimG<'a, esp32::timg::Freq20MHz, true>;
}
