// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2026.

use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::utilities::StaticRef;

register_structs! {
    RngRegisters {
        // RNG control register
        (0x000 => cr: ReadWrite<u32, CR::Register>),

        // RNG status register
        (0x004 => sr: ReadOnly<u32, SR::Register>),

        // RNG data register
        (0x008 => dr: ReadOnly<u32, DR::Register>),

        // RNG noice source control register
        (0x00C => nscr: ReadWrite<u32, NSCR::Register>),

        // RNG health test control register
        (0x010 => htcr: ReadWrite<u32, HTCR::Register>),

        (0x014 => @END),
    }
}

register_bitfields! [u32,
    CR [
        // RNG config lock
        CONFIGLOCK OFFSET(31) NUMBITS(1) [],

        // Conditioning soft reset
        CONDRST OFFSET(30) NUMBITS(1) [],

        RNG_CONFIG1 OFFSET(25) NUMBITS(6) [],

        // Clock divider factor
        CLKDIV OFFSET(19) NUMBITS(4) [],

        RNG_CONFIG2 OFFSET(15) NUMBITS(3) [],

        // NIST custom
        NISTC OFFSET(12) NUMBITS(1) [],

        RNG_CONFIG3 OFFSET(11) NUMBITS(4) [],

        // Auto reset disable
        ARDIS OFFSET(7) NUMBITS(1) [],

        // Clock error detection
        CED OFFSET(5) NUMBITS(1) [],

        // Interrupt enable
        IE OFFSET(3) NUMBITS(1) [],

        // True random number generator enable
        RNGEN OFFSET(2) NUMBITS(1) [],

    ],

    SR [
        // Seed error interrupt status
        SEIS OFFSET(6) NUMBITS(1) [],

        // Clock error interrupt status
        CEIS OFFSET(5) NUMBITS(1) [],

        // Seed error current status
        SECS OFFSET(2) NUMBITS(1) [],

        // Clock error current status
        CECS OFFSET(1) NUMBITS(1) [],

        // Data ready
        DRDY OFFSET(0) NUMBITS(1) [],
    ],

    DR [
        // Random data
        RNDATA OFFSET(31) NUMBITS(32) [],
    ],

    NSCR [
        EN_OSC6 OFFSET(17) NUMBITS(3) [],
        EN_OSC5 OFFSET(14) NUMBITS(3) [],
        EN_OSC4 OFFSET(11) NUMBITS(3) [],
        EN_OSC3 OFFSET(8) NUMBITS(3) [],
        EN_OSC2 OFFSET(5) NUMBITS(3) [],
        EN_OSC1 OFFSET(2) NUMBITS(3) [],
    ],

    HTCR [
        // Health test configuration
        HTCFG OFFSET(31) NUMBITS(32) [],
    ],
];

const RNG_BASE: StaticRef<RngRegisters> =
    unsafe { StaticRef::new(0x42020800 as *const RngRegisters) };

pub struct Rng {
    registers: StaticRef<RngRegisters>,
}

impl Rng {
    pub const fn new() -> Rng {
        Rng {
            registers: RNG_BASE,
        }
    }

    pub fn enable(&self) {
        self.registers.cr.write(CR::RNGEN::SET);
        kernel::debug!("RNG enabled");
    }
}
