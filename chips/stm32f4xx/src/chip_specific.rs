// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL.
//
// Author: Ioan-Cristian CÎRSTEA <ioan.cirstea@oxidos.io>

#![deny(missing_docs)]
#![deny(dead_code)]

//! This module contains all chip-specific code.
//!
//! Some models in the STM32F4 family may have additional features, while others not. Or they can
//! operate internally in different ways for the same feature. This crate provides all the
//! chip-specific crate to be used by others modules in this crate.

/// Clock-related constants for specific chips
pub mod clock_constants {
    // Internal constants prefixed by double underscores are created to avoid the need to duplicate
    // documentation comments

    /// PLL-related constants for specific chips
    pub mod pll_constants {
        // STM32F401 supports frequency down to 24MHz. All other chips in the F4 family down to
        // 13MHz.
        #[cfg(not(feature = "stm32f401"))]
        const __PLL_MIN_FREQ_MHZ: usize = 13;
        #[cfg(feature = "stm32f401")]
        const __PLL_MIN_FREQ_MHZ: usize = 24;

        /// Minimum PLL frequency in MHz
        pub const PLL_MIN_FREQ_MHZ: usize = __PLL_MIN_FREQ_MHZ;
    }

    #[cfg(any(feature = "stm32f412"))]
    const __APB1_FREQUENCY_LIMIT_MHZ: usize = 50;
    #[cfg(any(feature = "stm32f429", feature = "stm32f446"))]
    const __APB1_FREQUENCY_LIMIT_MHZ: usize = 45;
    #[cfg(any(feature = "stm32f401"))]
    const __APB1_FREQUENCY_LIMIT_MHZ: usize = 42;

    /// Maximum allowed APB1 frequency in MHz
    pub const APB1_FREQUENCY_LIMIT_MHZ: usize = __APB1_FREQUENCY_LIMIT_MHZ;

    /// Maximum allowed APB2 frequency in MHz
    // APB2 frequency limit is twice the APB1 frequency limit
    pub const APB2_FREQUENCY_LIMIT_MHZ: usize = APB1_FREQUENCY_LIMIT_MHZ << 1;

    #[cfg(any(feature = "stm32f412"))]
    const __SYS_CLOCK_FREQUENCY_LIMIT_MHZ: usize = 100;
    #[cfg(any(feature = "stm32f429", feature = "stm32f446"))]
    const __SYS_CLOCK_FREQUENCY_LIMIT_MHZ: usize = 168;
    #[cfg(any(feature = "stm32f401"))]
    const __SYS_CLOCK_FREQUENCY_LIMIT_MHZ: usize = 84;

    /// Maximum allowed system clock frequency in MHz
    pub const SYS_CLOCK_FREQUENCY_LIMIT_MHZ: usize = __SYS_CLOCK_FREQUENCY_LIMIT_MHZ;
}

/// Chip-specific flash code
pub mod flash_specific {
    #[cfg(any(
        feature = "stm32f401",
        feature = "stm32f412",
        feature = "stm32f429",
        feature = "stm32f446"
    ))]
    #[derive(Copy, Clone, PartialEq, Debug)]
    /// Enum representing all the possible values for the flash latency
    pub(crate) enum FlashLatency {
        /// 0 wait cycles
        Latency0,
        /// 1 wait cycle
        Latency1,
        /// 2 wait cycles
        Latency2,
        /// 3 wait cycles
        Latency3,
        /// 4 wait cycles
        Latency4,
        /// 5 wait cycles
        Latency5,
        /// 6 wait cycles
        Latency6,
        /// 7 wait cycles
        Latency7,
        /// 8 wait cycles
        Latency8,
        /// 9 wait cycles
        Latency9,
        /// 10 wait cycles
        Latency10,
        /// 11 wait cycles
        Latency11,
        /// 12 wait cycles
        Latency12,
        /// 13 wait cycles
        Latency13,
        /// 14 wait cycles
        Latency14,
        /// 15 wait cycles
        Latency15,
    }

    // The number of wait cycles depends on two factors: system clock frequency and the supply
    // voltage. Currently, this method assumes 2.7-3.6V voltage supply (default value).
    // TODO: Take into the account the power supply
    //
    // The number of wait states varies from chip to chip.
    pub(crate) fn get_number_wait_cycles_based_on_frequency(frequency_mhz: usize) -> FlashLatency {
        // feature = "stm32f401"
        // feature = "stm32f429"
        // feature = "stm32f446"
        #[cfg(not(feature = "stm32f412"))]
        {
            match frequency_mhz {
                0..=30 => FlashLatency::Latency0,
                31..=60 => FlashLatency::Latency1,
                61..=90 => FlashLatency::Latency2,
                91..=120 => FlashLatency::Latency3,
                121..=150 => FlashLatency::Latency4,
                _ => FlashLatency::Latency5,
            }
        }
        #[cfg(any(feature = "stm32f412"))]
        {
            match frequency_mhz {
                0..=30 => FlashLatency::Latency0,
                31..=64 => FlashLatency::Latency1,
                65..=90 => FlashLatency::Latency2,
                _ => FlashLatency::Latency3,
            }
        }
    }

    pub(crate) fn convert_register_to_enum(flash_latency_register: u32) -> FlashLatency {
        #[cfg(any(
            feature = "stm32f401",
            feature = "stm32f412",
            feature = "stm32f429",
            feature = "stm32f446"
        ))]
        match flash_latency_register {
            0 => FlashLatency::Latency0,
            1 => FlashLatency::Latency1,
            2 => FlashLatency::Latency2,
            3 => FlashLatency::Latency3,
            4 => FlashLatency::Latency4,
            5 => FlashLatency::Latency5,
            6 => FlashLatency::Latency6,
            7 => FlashLatency::Latency7,
            8 => FlashLatency::Latency8,
            9 => FlashLatency::Latency9,
            10 => FlashLatency::Latency10,
            11 => FlashLatency::Latency11,
            12 => FlashLatency::Latency12,
            13 => FlashLatency::Latency13,
            14 => FlashLatency::Latency14,
            // The hardware allows 4-bit latency values
            _ => FlashLatency::Latency15,
        }
    }
}
