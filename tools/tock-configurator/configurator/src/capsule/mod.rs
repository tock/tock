// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024
// Copyright OxidOS Automotive SRL 2024
//
// Author: Irina Nita <irina.nita@oxidos.io>
// Author: Darius Jipa <darius.jipa@oxidos.io>

use cursive::views::LinearLayout;
use parse::peripherals::Chip;
use std::rc::Rc;

pub(crate) mod alarm;
pub(crate) mod ble;
pub(crate) mod console;
pub(crate) mod flash;
pub(crate) mod gpio;
pub(crate) mod i2c;
pub(crate) mod lsm303agr;
pub(crate) mod rng;
pub(crate) mod spi;
pub(crate) mod temperature;

pub trait ConfigMenu: std::fmt::Debug {
    fn config<C: Chip + 'static + serde::ser::Serialize>(chip: Rc<C>) -> LinearLayout;
}
