// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024
// Copyright OxidOS Automotive SRL 2024
//
// Author: Irina Nita <irina.nita@oxidos.io>
// Author: Darius Jipa <darius.jipa@oxidos.io>

//! Traits that a chip's peripherals found in OxidOS must implement in order to be
//! used by the configurator.

pub mod chip;
pub use chip::*;

pub mod gpio;
pub use gpio::*;

pub mod timer;
pub use timer::*;

pub mod uart;
pub use uart::*;

pub mod spi;
pub use spi::*;

pub mod i2c;
pub use i2c::*;

pub mod ble;
pub use ble::*;

pub mod flash;
pub use flash::*;

pub mod temp;
pub use temp::*;

pub mod rng;
pub use rng::*;
