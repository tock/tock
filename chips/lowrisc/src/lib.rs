//! Implementations for generic LowRISC peripherals.

// Feature required with newer versions of rustc (at least 2020-10-25).
#![feature(const_mut_refs)]
#![no_std]
#![crate_name = "lowrisc"]
#![crate_type = "rlib"]

pub mod csrng;
pub mod flash_ctrl;
pub mod gpio;
pub mod hmac;
pub mod i2c;
pub mod otbn;
pub mod padctrl;
pub mod pwrmgr;
pub mod spi_host;
pub mod uart;
pub mod usbdev;
pub mod virtual_otbn;
