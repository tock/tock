// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL.
//
// Author: Ioan-Cristian CÎRSTEA <ioan.cirstea@oxidos.io>

#![deny(missing_docs)]
#![deny(dead_code)]
#![deny(unused_imports)]

//! This module contains all chip-specific code.
//!
//! Some models in the STM32F4 family may have additional features, while others not. Or they can
//! operate internally in different ways for the same feature. This crate provides all the
//! chip-specific crate to be used by others modules in this crate.

/// Clock-related constants for specific chips
pub mod clock_constants {
    /// PLL-related constants for specific chips
    pub trait PllConstants {
        /// PLL minimum frequency in MHz
        const MIN_FREQ_MHZ: usize;
        /// PLL maximum frequency in MHz
        // All boards support PLL frequencies up to 216MHz
        const MAX_FREQ_MHZ: usize = 216;
    }

    /// Generic clock constants
    pub trait ClockConstants {
        /// Maximum allowed APB1 frequency in MHz
        const APB1_FREQUENCY_LIMIT_MHZ: usize;
        /// Maximum allowed APB2 frequency in MHz
        // APB2 frequency limit is twice the APB1 frequency limit
        const APB2_FREQUENCY_LIMIT_MHZ: usize = Self::APB1_FREQUENCY_LIMIT_MHZ << 1;
        /// Maximum allowed system clock frequency in MHz
        const SYS_CLOCK_FREQUENCY_LIMIT_MHZ: usize;
    }
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
    #[cfg(any(feature = "stm32f401", feature = "stm32f429", feature = "stm32f446"))]
    pub(crate) fn get_number_wait_cycles_based_on_frequency(frequency_mhz: usize) -> FlashLatency {
        match frequency_mhz {
            0..=30 => FlashLatency::Latency0,
            31..=60 => FlashLatency::Latency1,
            61..=90 => FlashLatency::Latency2,
            91..=120 => FlashLatency::Latency3,
            121..=150 => FlashLatency::Latency4,
            _ => FlashLatency::Latency5,
        }
    }

    #[cfg(feature = "stm32f412")]
    pub(crate) fn get_number_wait_cycles_based_on_frequency(frequency_mhz: usize) -> FlashLatency {
        match frequency_mhz {
            0..=30 => FlashLatency::Latency0,
            31..=64 => FlashLatency::Latency1,
            65..=90 => FlashLatency::Latency2,
            _ => FlashLatency::Latency3,
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
