// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Chip support for the E310-G002 from SiFive.

#![no_std]

pub use e310x::{chip, clint, gpio, plic, prci, pwm, rtc, uart, watchdog};

pub mod interrupt_service;
mod interrupts;
