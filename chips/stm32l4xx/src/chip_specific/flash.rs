// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Author: Kamil Duljas <kamil.duljas@gmail.com>

//! Chip-specific flash code

use core::fmt::Debug;

pub trait FlashChipSpecific {
    type FlashLatency: RegisterToFlashLatency + Clone + Copy + PartialEq + Debug + Into<u32>;

    // The number of wait cycles depends on two factors: system clock frequency and the supply
    // voltage.
    fn get_number_wait_cycles_based_on_frequency_and_voltage(
        frequency_mhz: usize,
        vos: usize,
    ) -> Self::FlashLatency;
}

pub trait RegisterToFlashLatency {
    fn convert_register_to_enum(flash_latency_register: u32) -> Self;
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum FlashLatency5 {
    Latency0,
    Latency1,
    Latency2,
    Latency3,
    Latency4,
    Latency5,
}

impl RegisterToFlashLatency for FlashLatency5 {
    fn convert_register_to_enum(flash_latency_register: u32) -> Self {
        match flash_latency_register {
            0 => Self::Latency0,
            1 => Self::Latency1,
            2 => Self::Latency2,
            3 => Self::Latency3,
            4 => Self::Latency4,
            _ => Self::Latency5,
        }
    }
}

impl From<FlashLatency5> for u32 {
    fn from(val: FlashLatency5) -> Self {
        val as u32
    }
}
