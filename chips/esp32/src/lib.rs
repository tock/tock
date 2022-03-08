//! Drivers and chip support for Espressif ESP32 boards.

#![no_std]
#![crate_name = "esp32"]
#![crate_type = "rlib"]

pub mod gpio;
pub mod rtc_cntl;
pub mod timg;
pub mod uart;
