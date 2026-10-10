// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Pulse Width Modulation (PWM) driver.

use kernel::utilities::StaticRef;
use kernel::utilities::registers::interfaces::Writeable;
use kernel::utilities::registers::{ReadWrite, register_bitfields, register_structs};

register_structs! {
    pub PwmRegisters {
        /// PWM Configuration Register
        (0x000 => cfg: ReadWrite<u32, cfg::Register>),
        (0x004 => _reserved0),
        /// Counter Register
        (0x008 => count: ReadWrite<u32>),
        (0x00c => _reserved1),
        /// Scaled Halfword Counter Register
        (0x010 => pwms: ReadWrite<u32>),
        (0x014 => _reserved2),
        /// Compare Register
        (0x020 => cmp0: ReadWrite<u32>),
        /// Compare Register
        (0x024 => cmp1: ReadWrite<u32>),
        /// Compare Register
        (0x028 => cmp2: ReadWrite<u32>),
        /// Compare Register
        (0x02c => cmp3: ReadWrite<u32>),
        (0x030 => @END),
    }
}

register_bitfields![u32,
    cfg [
        cmp3ip OFFSET(31) NUMBITS(1) [],
        cmp2ip OFFSET(30) NUMBITS(1) [],
        cmp1ip OFFSET(29) NUMBITS(1) [],
        cmp0ip OFFSET(28) NUMBITS(1) [],
        cmp3gang OFFSET(27) NUMBITS(1) [],
        cmp2gang OFFSET(26) NUMBITS(11) [],
        cmp1gang OFFSET(25) NUMBITS(1) [],
        cmp0gang OFFSET(24) NUMBITS(1) [],
        cmp3center OFFSET(19) NUMBITS(1) [],
        cmp2center OFFSET(18) NUMBITS(1) [],
        cmp1center OFFSET(17) NUMBITS(1) [],
        cmp0center OFFSET(16) NUMBITS(1) [],
        enoneshot OFFSET(13) NUMBITS(1) [],
        enalways OFFSET(12) NUMBITS(1) [],
        deglitch OFFSET(10) NUMBITS(1) [],
        zerocmp OFFSET(9) NUMBITS(1) [],
        sticky OFFSET(8) NUMBITS(1) [],
        scale OFFSET(0) NUMBITS(4) []
    ]
];

pub struct Pwm {
    registers: StaticRef<PwmRegisters>,
}

impl Pwm {
    pub const fn new(base: StaticRef<PwmRegisters>) -> Pwm {
        Pwm { registers: base }
    }

    /// Disable the PWM so it does not generate interrupts.
    pub fn disable(&self) {
        let regs = self.registers;

        // Turn the interrupt compare off so we don't get any RTC interrupts.
        regs.cfg.write(cfg::enalways::CLEAR + cfg::enoneshot::CLEAR);

        // Set the comparitors high to make sure we don't get interrupts
        regs.cmp0.set(0x0000_FFFF);
        regs.cmp1.set(0x0000_FFFF);
        regs.cmp2.set(0x0000_FFFF);
        regs.cmp3.set(0x0000_FFFF);
    }
}
