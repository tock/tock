// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Drivers and chip support for ESP32-C3.

#![no_std]

pub mod chip;
pub mod intc;
pub mod interrupts;
pub mod rng;
pub mod sysreg;

pub mod timg {
    pub use esp32::timg::{ClockSource, TIMG0_BASE, TIMG1_BASE};
    pub type TimG<'a> = esp32::timg::TimG<'a, esp32::timg::Freq20MHz, true>;
}
