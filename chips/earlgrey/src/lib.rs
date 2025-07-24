// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Drivers and chip support for EarlGrey.

#![no_std]
// `registers/rv_plic_regs` has many register definitions in `register_structs()!`
// and requires a deeper recursion limit than the default to fully expand.
#![recursion_limit = "512"]

pub mod chip_config;
pub mod pinmux_config;

mod interrupts;

pub mod aes;
pub mod aon_timer;
pub mod chip;
pub mod csrng;
pub mod epmp;
pub mod flash_ctrl;
pub mod gpio;
pub mod hmac;
pub mod i2c;
pub mod otbn;
pub mod pinmux;
pub mod plic;
pub mod pwrmgr;
pub mod registers;
pub mod spi_host;
pub mod timer;
pub mod uart;
pub mod usbdev;
