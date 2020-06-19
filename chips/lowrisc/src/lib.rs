//! Implementations for generic LowRISC peripherals.

#![feature(const_fn)]
#![no_std]
#![crate_name = "lowrisc"]
#![crate_type = "rlib"]

pub mod gpio;
pub mod hmac;
pub mod i2c;
pub mod pwrmgr;
pub mod uart;
pub mod usbdev;
