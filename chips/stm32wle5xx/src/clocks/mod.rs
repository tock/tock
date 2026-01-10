// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! The clock module for STM32WLE5xx chips.
//!
//! This is highly similar to the one for STM32L4xx chips. This clock
//! implementation provides the minimal functionality required to enable
//! peripherals and configure speeds (as tested for I2C and UART). This
//! is still highly a work in progress and documentation comments here
//! describing the usage will be updated as development continues.

pub mod clocks;
pub mod hse;
pub mod hsi;
pub mod msi;
pub mod phclk;
pub mod pll;

pub use crate::clocks::clocks::{Clocks, Stm32wle5xxClocks};
