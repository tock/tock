// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL.
//
// Author: Ioan-Cristian CÎRSTEA <ioan.cirstea@oxidos.io>

pub mod clock_constants {
    pub mod pll_constants {
        pub const PLL_MIN_FREQ_MHZ: usize = if cfg!(not(feature = "stm32f401")) {
            13
        } else {
            24
        };
    }

    pub const APB1_FREQUENCY_LIMIT_MHZ: usize = if cfg!(any(
        feature = "stm32f410",
        feature = "stm32f411",
        feature = "stm32f412",
        feature = "stm32f413",
        feature = "stm32f423"
    )) {
        50
    } else if cfg!(any(
        feature = "stm32f427",
        feature = "stm32f429",
        feature = "stm32f437",
        feature = "stm32f439",
        feature = "stm32f446",
        feature = "stm32f469",
        feature = "stm32f479",
    )) {
        45
    } else {
        //feature = "stm32f401",
        //feature = "stm32f405",
        //feature = "stm32f407",
        //feature = "stm32f415",
        //feature = "stm32f417"
        42
    };

    // APB2 frequency limit is twice the APB1 frequency limit
    pub const APB2_FREQUENCY_LIMIT_MHZ: usize = APB1_FREQUENCY_LIMIT_MHZ << 1;

    pub const SYS_CLOCK_FREQUENCY_LIMIT_MHZ: usize = if cfg!(any(
        feature = "stm32f410",
        feature = "stm32f411",
        feature = "stm32f412",
        feature = "stm32f413",
        feature = "stm32f423"
    )) {
        100
    } else if cfg!(any(
        feature = "stm32f405",
        feature = "stm32f407",
        feature = "stm32f415",
        feature = "stm32f417",
        feature = "stm32f427",
        feature = "stm32f429",
        feature = "stm32f437",
        feature = "stm32f439",
        feature = "stm32f446",
        feature = "stm32f469",
        feature = "stm32f479"
    )) {
        // TODO: Some of these models support overdrive model. Change this constant when overdrive support
        // is added.
        168
    } else {
        //feature = "stm32f401"
        84
    };
}
