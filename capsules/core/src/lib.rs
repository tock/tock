// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2023.

#![forbid(unsafe_code)]
#![no_std]

// pub mod test;

#[macro_use]
pub mod stream;

pub mod adc;
pub mod alarm;
pub mod button;
pub mod console;
pub mod console_ordered;
pub mod driver;
pub mod gpio;
pub mod i2c_master;
pub mod i2c_master_slave_combo;
pub mod i2c_master_slave_driver;
pub mod led;
pub mod low_level_debug;
pub mod process_console;
pub mod rng;
pub mod spi_controller;
pub mod spi_peripheral;
pub mod virtualizers;
