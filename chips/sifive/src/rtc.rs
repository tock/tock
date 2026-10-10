// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Real Time Clock (RTC) driver.

use kernel::utilities::StaticRef;
use kernel::utilities::registers::interfaces::Writeable;
use kernel::utilities::registers::{ReadWrite, register_bitfields, register_structs};

register_structs! {
    pub RtcRegisters {
        /// RTC Configuration Register
        (0x000 => rtccfg: ReadWrite<u32, rtccfg::Register>),
        (0x004 => _reserved1),
        /// RTC Counter Low Register
        (0x008 => rtclo: ReadWrite<u32, rtclo::Register>),
        /// RTC Counter High Register
        (0x00c => rtchi: ReadWrite<u32>),
        /// RTC Scaled Counter Register
        (0x010 => rtcs: ReadWrite<u32>),
        (0x014 => _reserved2),
        /// RTC Compare Register
        (0x020 => rtccmp: ReadWrite<u32, rtccmp::Register>),
        (0x024 => @END),
    }
}

register_bitfields![u32,
    rtccfg [
        cmpip OFFSET(28) NUMBITS(1) [],
        enalways OFFSET(12) NUMBITS(1) [],
        scale OFFSET(0) NUMBITS(4) []
    ],
    rtclo [
        rtclo OFFSET(0) NUMBITS(32) []
    ],
    rtchi [
        rtchi OFFSET(0) NUMBITS(16) []
    ],
    rtccmp [
        rtccmp OFFSET(0) NUMBITS(32) []
    ]
];

pub struct Rtc {
    registers: StaticRef<RtcRegisters>,
}

impl Rtc {
    pub const fn new(base: StaticRef<RtcRegisters>) -> Rtc {
        Rtc { registers: base }
    }

    /// Disable the RTC so it does not generate interrupts.
    pub fn disable(&self) {
        let regs = self.registers;

        // Turn the interrupt compare off so we don't get any RTC interrupts.
        regs.rtccfg.write(rtccfg::enalways::CLEAR);

        // Set the compare time to as large as possible
        regs.rtccmp.set(0xFFFF_FFFF);
    }
}
