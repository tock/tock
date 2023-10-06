// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL.
//
// Author: Ioan-Cristian CÎRSTEA <ioan.cirstea@oxidos.io>

//! STM32F401 specifications

use stm32f4xx::chip_specific::clock_constants::{PllConstants, SystemClockConstants};
use stm32f4xx::chip_specific::flash::{FlashChipSpecific, FlashLatency16};

pub enum Stm32f401Specs {}

impl PllConstants for Stm32f401Specs {
    const MIN_FREQ_MHZ: usize = 24;
}

impl SystemClockConstants for Stm32f401Specs {
    const APB1_FREQUENCY_LIMIT_MHZ: usize = 42;
    const SYS_CLOCK_FREQUENCY_LIMIT_MHZ: usize = 84;
}

impl FlashChipSpecific for Stm32f401Specs {
    type FlashLatency = FlashLatency16;

    fn get_number_wait_cycles_based_on_frequency(frequency_mhz: usize) -> Self::FlashLatency {
        match frequency_mhz {
            0..=30 => Self::FlashLatency::Latency0,
            31..=60 => Self::FlashLatency::Latency1,
            61..=90 => Self::FlashLatency::Latency2,
            91..=120 => Self::FlashLatency::Latency3,
            121..=150 => Self::FlashLatency::Latency4,
            _ => Self::FlashLatency::Latency5,
        }
    }
}
