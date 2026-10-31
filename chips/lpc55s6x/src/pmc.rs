// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Power Management Controller (PMC) for the LPC55S6x family.
//!
//! Only the run-mode power-down configuration needed by the drivers in this
//! crate is exposed.

use kernel::utilities::StaticRef;
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{ReadOnly, WriteOnly, register_bitfields, register_structs};

register_structs! {
    /// PMC
    PmcRegisters {
        (0x000 => _reserved0),
        /// Controls the power to various analog blocks.
        (0x0B8 => pdruncfg0: ReadOnly<u32, PDRUNCFG0::Register>),
        (0x0BC => _reserved1),
        /// Writing ones to this register clears the corresponding bits in PDRUNCFG0.
        (0x0C8 => pdruncfgclr0: WriteOnly<u32, PDRUNCFG0::Register>),
        (0x0CC => @END),
    }
}

register_bitfields![u32,
PDRUNCFG0 [
    /// Controls power to all True Random Number Generator (TRNG) clock sources.
    PDEN_RNG OFFSET(22) NUMBITS(1) [
        /// TRNG clocks are powered.
        Powered = 0,
        /// TRNG clocks are powered-down.
        PoweredDown = 1
    ]
]
];

const PMC_BASE: StaticRef<PmcRegisters> =
    unsafe { StaticRef::new(0x40020000 as *const PmcRegisters) };

pub struct Pmc {
    registers: StaticRef<PmcRegisters>,
}

impl Pmc {
    pub const fn new() -> Pmc {
        Pmc {
            registers: PMC_BASE,
        }
    }

    /// Power up the ring oscillators used as entropy sources by the RNG.
    pub fn power_up_rng(&self) {
        self.registers.pdruncfgclr0.write(PDRUNCFG0::PDEN_RNG::SET);
    }

    pub fn is_rng_powered(&self) -> bool {
        self.registers
            .pdruncfg0
            .matches_all(PDRUNCFG0::PDEN_RNG::Powered)
    }
}
