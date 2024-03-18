// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Implementations for generic SiFive MCU peripherals.

#![no_std]
#![feature(generic_const_exprs)]
#![crate_name = "sifive"]
#![crate_type = "rlib"]

pub const MAX_THREADS: usize = 2;

pub mod clint;
pub mod gpio;
pub mod plic;
pub mod prci;
pub mod pwm;
pub mod rtc;
pub mod uart;
pub mod watchdog;
