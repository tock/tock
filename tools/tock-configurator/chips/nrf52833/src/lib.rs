// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024
// Copyright OxidOS Automotive SRL 2024
//
// Author: Irina Nita <irina.nita@oxidos.io>
// Author: Darius Jipa <darius.jipa@oxidos.io>

pub mod ble;
pub mod chip;
pub mod flash;
pub mod gpio;
pub mod peripherals;
pub mod rng;
pub mod temperature;
pub mod timer;
pub mod twi;
pub mod uart;

pub use ble::*;
pub use chip::*;
pub use flash::*;
pub use gpio::*;
pub use peripherals::*;
pub use rng::*;
pub use temperature::*;
pub use timer::*;
pub use twi::*;
pub use uart::*;
