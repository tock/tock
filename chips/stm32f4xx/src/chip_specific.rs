// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL.

pub mod pll_constants {
    pub const PLL_MIN_FREQ_MHZ: usize = if cfg!(not(feature = "stm32f401")) {
        13
    } else {
        24
    };
}
