// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Author: Kamil Duljas <kamil.duljas@gmail.com>

pub mod clocks;
pub mod hsi;
pub mod msi;
pub mod phclk;
pub mod pll;
pub use crate::clocks::clocks::{Clocks, Stm32l4Clocks};
