// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Peripheral implementations for the nRF5340 network core.

#![no_std]

pub mod chip;
pub mod clock;
pub mod crt1;
pub mod gpio;
pub mod peripheral_interrupts;

pub use nrf5x::{ble_radio, pinmux, rtc, uarte};
