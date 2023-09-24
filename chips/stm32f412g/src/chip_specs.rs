// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL.
//
// Author: Ioan-Cristian CÎRSTEA <ioan.cirstea@oxidos.io>

//! STM32F412 specifications

use stm32f4xx::chip_specific::clock_constants::{PllConstants, SystemClockConstants};
use stm32f4xx::chip_specific::flash::{FlashChipSpecific, FlashLatency16};

pub enum Stm32f412Specs {}

impl PllConstants for Stm32f412Specs {
    const MIN_FREQ_MHZ: usize = 13;
}

impl SystemClockConstants for Stm32f412Specs {
    const APB1_FREQUENCY_LIMIT_MHZ: usize = 50;
    const SYS_CLOCK_FREQUENCY_LIMIT_MHZ: usize = 100;
}

impl FlashChipSpecific for Stm32f412Specs {
    type FlashLatency = FlashLatency16;

    fn get_number_wait_cycles_based_on_frequency(frequency_mhz: usize) -> Self::FlashLatency {
        match frequency_mhz {
            0..=30 => Self::FlashLatency::Latency0,
            31..=64 => Self::FlashLatency::Latency1,
            65..=90 => Self::FlashLatency::Latency2,
            _ => Self::FlashLatency::Latency3,
        }
    }
}
