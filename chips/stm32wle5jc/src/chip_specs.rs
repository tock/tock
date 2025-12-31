// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

// (TODO): THESE VALUES ARE NOT FILLED IN...JUST COPIED FROM F4 for now.
use stm32wle5xx::chip_specific::clock_constants::{PllConstants, SystemClockConstants};

pub enum Stm32wle5jcSpecs {}

impl PllConstants for Stm32wle5jcSpecs {
    const MIN_FREQ_MHZ: usize = 13;
}

impl SystemClockConstants for Stm32wle5jcSpecs {
    const APB1_FREQUENCY_LIMIT_MHZ: usize = 45;
    const SYS_CLOCK_FREQUENCY_LIMIT_MHZ: usize = 168;
}
