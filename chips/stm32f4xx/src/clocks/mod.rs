// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL.
//
// Author: Ioan-Cristian CÎRSTEA <ioan.cirstea@oxidos.io>

pub mod clocks;
pub mod hsi;
pub mod pll;

/// Clock various limits
pub mod limits {
    pub use crate::clocks::pll::limits::*;
    pub use crate::chip_specific::clock_constants::APB1_FREQUENCY_LIMIT_MHZ;
    pub use crate::chip_specific::clock_constants::APB2_FREQUENCY_LIMIT_MHZ;
    pub use crate::chip_specific::clock_constants::SYS_CLOCK_FREQUENCY_LIMIT_MHZ;
}

pub use crate::clocks::clocks::tests;
pub use crate::clocks::clocks::Clocks;
