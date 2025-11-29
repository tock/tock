// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

//! CYW4343x driver
//!
//! Datasheet: <https://www.mouser.com/datasheet/2/196/Infineon_CYW43439_DataSheet_v03_00_EN-3074791.pdf>
//! Infineon's WHD (WiFi Host Driver) documentation: <https://infineon.github.io/wifi-host-driver/html/index.html>
//!
//! The implementation consists of two main components:
//! - driver: handles higher-level WLAN packet transmission using the bus
//! - bus: handles communicating with the WiFi chip over SDIO or gSPI (configuring protocol-specific registers,
//!   loading the firmware, sending WLAN data from the driver)

mod bus;
mod constants;
mod driver;
mod macros;
mod sdpcm;

pub use bus::spi as spi_bus;
pub use bus::{CYW4343xBus, CYW4343xBusClient};
pub use driver::CYW4343x;
