// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Author: Kamil Duljas <kamil.duljas@gmail.com>

//! STM32L476 specifications

use stm32l4xx::chip_specific::clock_constants::{PllConstants, SystemClockConstants};
use stm32l4xx::chip_specific::flash::{FlashChipSpecific, FlashLatency5};

pub enum Stm32l476Specs {}

impl PllConstants for Stm32l476Specs {}

impl SystemClockConstants for Stm32l476Specs {}

impl FlashChipSpecific for Stm32l476Specs {
    type FlashLatency = FlashLatency5;

    fn get_number_wait_cycles_based_on_frequency_and_voltage(
        frequency_mhz: usize,
        vos: usize,
    ) -> Self::FlashLatency {
        match vos {
            1 => match frequency_mhz {
                0..=16 => Self::FlashLatency::Latency0,
                17..=32 => Self::FlashLatency::Latency1,
                33..=48 => Self::FlashLatency::Latency2,
                49..=64 => Self::FlashLatency::Latency3,
                65..=80 => Self::FlashLatency::Latency4,
                _ => Self::FlashLatency::Latency5,
            },
            2 => match frequency_mhz {
                0..=6 => Self::FlashLatency::Latency0,
                7..=12 => Self::FlashLatency::Latency1,
                13..=18 => Self::FlashLatency::Latency2,
                19..=26 => Self::FlashLatency::Latency3,
                _ => Self::FlashLatency::Latency5,
            },
            _ => panic!("Unexpected VOS!"),
        }
    }
}
