// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2024
//
// Author: Irina Nita <irina.nita@oxidos.io>
// Author: Darius Jipa <darius.jipa@oxidos.io>

//  TODO: Using separate structs/enums only for serialization and trait implementation
//  purposes is making the code hard to read and hard to scale. A more generalized method
//  (e.g. using a JSON of a chip configuration that deserializes into a single abstract struct)
//  would be a better candidate. The documentation on the `chips` directory
//  provided better insight on how these JSONs could look like.

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
