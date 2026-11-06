// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Connection of the 32.768 kHz crystal oscillator (LFXO).
//!
//! The crystal sits on XL1 (P0.00) and XL2 (P0.01). Only secure code can hand
//! those pins to the oscillator, so a non-secure kernel relies on its secure
//! stage for this. Either core can then run its LFCLK from the LFXO.

use kernel::utilities::StaticRef;
use kernel::utilities::registers::interfaces::{ReadWriteable, Writeable};
use kernel::utilities::registers::{ReadWrite, register_bitfields, register_structs};

use crate::network::{PinCnf, PinCnfRegisters};

register_structs! {
    pub OscillatorsRegisters {
        (0x000 => _reserved0),
        (0x6D0 => xosc32ki_intcap: ReadWrite<u32, IntCap::Register>),
        (0x6D4 => @END),
    }
}

register_bitfields! [u32,
    IntCap [
        INTCAP OFFSET(0) NUMBITS(2) [
            External = 0,
            C6pF = 1,
            C7pF = 2,
            C9pF = 3
        ]
    ]
];

/// The load capacitance of the crystal.
pub enum LoadCapacitance {
    External,
    Internal6pF,
    Internal7pF,
    Internal9pF,
}

pub struct Lfxo {
    oscillators: StaticRef<OscillatorsRegisters>,
    port0: StaticRef<PinCnfRegisters>,
}

impl Lfxo {
    pub const fn new(
        oscillators: StaticRef<OscillatorsRegisters>,
        port0: StaticRef<PinCnfRegisters>,
    ) -> Self {
        Self { oscillators, port0 }
    }

    /// Hand XL1 and XL2 to the oscillator, which then sees `capacitance`.
    /// This needs secure state.
    pub fn connect(&self, capacitance: LoadCapacitance) {
        self.oscillators.xosc32ki_intcap.write(match capacitance {
            LoadCapacitance::External => IntCap::INTCAP::External,
            LoadCapacitance::Internal6pF => IntCap::INTCAP::C6pF,
            LoadCapacitance::Internal7pF => IntCap::INTCAP::C7pF,
            LoadCapacitance::Internal9pF => IntCap::INTCAP::C9pF,
        });
        for pin_cnf in &self.port0.pin_cnf[..2] {
            pin_cnf.modify(PinCnf::MCUSEL::Peripheral);
        }
    }
}
