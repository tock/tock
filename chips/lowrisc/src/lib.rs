// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Implementations for generic LowRISC peripherals.

#![no_std]

pub mod aon_timer;
pub mod csrng;
pub mod flash_ctrl;
pub mod gpio;
pub mod hmac;
pub mod i2c;
pub mod otbn;
pub mod padctrl;
pub mod pwrmgr;
pub mod registers;
pub mod rsa;
pub mod spi_host;
pub mod uart;
pub mod usbdev;
pub mod virtual_otbn;
