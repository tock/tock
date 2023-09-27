// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL.
//
// Author: Ioan-Cristian CÎRSTEA <ioan.cirstea@oxidos.io>

//! Chip-specific flash code

use core::fmt::Debug;

pub trait FlashChipSpecific {
    type FlashLatency: RegisterToFlashLatency + Clone + Copy + PartialEq + Debug + Into<u32>;

    // The number of wait cycles depends on two factors: system clock frequency and the supply
    // voltage. Currently, this method assumes 2.7-3.6V voltage supply (default value).
    // TODO: Take into account the power supply
    //
    // The number of wait cycles varies from chip to chip
    fn get_number_wait_cycles_based_on_frequency(frequency_mhz: usize) -> Self::FlashLatency;
}

pub trait RegisterToFlashLatency {
    fn convert_register_to_enum(flash_latency_register: u32) -> Self;
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum FlashLatency16 {
    Latency0,
    Latency1,
    Latency2,
    Latency3,
    Latency4,
    Latency5,
    Latency6,
    Latency7,
    Latency8,
    Latency9,
    Latency10,
    Latency11,
    Latency12,
    Latency13,
    Latency14,
    Latency15,
}

impl RegisterToFlashLatency for FlashLatency16 {
    fn convert_register_to_enum(flash_latency_register: u32) -> Self {
        match flash_latency_register {
            0 => Self::Latency0,
            1 => Self::Latency1,
            2 => Self::Latency2,
            3 => Self::Latency3,
            4 => Self::Latency4,
            5 => Self::Latency5,
            6 => Self::Latency6,
            7 => Self::Latency7,
            8 => Self::Latency8,
            9 => Self::Latency9,
            10 => Self::Latency10,
            11 => Self::Latency11,
            12 => Self::Latency12,
            13 => Self::Latency13,
            14 => Self::Latency14,
            // The hardware supports 4-bit flash latency
            _ => Self::Latency15,
        }
    }
}

impl Into<u32> for FlashLatency16 {
    fn into(self) -> u32 {
        self as u32
    }
}
