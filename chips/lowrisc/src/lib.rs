//! Implementations for generic LowRISC peripherals.

#![feature(const_fn, const_mut_refs)]
#![no_std]
#![crate_name = "lowrisc"]
#![crate_type = "rlib"]

pub mod flash_ctrl;
pub mod gpio;
pub mod hmac;
pub mod i2c;
pub mod padctrl;
pub mod pwrmgr;
pub mod uart;
pub mod usbdev;
