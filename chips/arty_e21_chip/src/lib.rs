// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Drivers and chip support for the E21 soft core.

#![no_std]

mod interrupts;

pub mod chip;
pub mod clint;
pub mod gpio;
pub mod uart;
