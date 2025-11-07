// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

#![no_std]
#![forbid(unsafe_code)]

#[macro_use]
mod utils;
mod bus;
mod component;
mod driver;
mod sdpcm;

pub use bus::spi as spi_bus;
pub use bus::{CYW4343xBus, CYW4343xBusClient};
pub use component::*;
pub use driver::CYW4343x;
