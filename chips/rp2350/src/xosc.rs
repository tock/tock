// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

use kernel::utilities::registers::interfaces::{ReadWriteable, Readable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::utilities::StaticRef;

register_structs! {
    /// Controls the crystal oscillator
    XoscRegisters {
        /// Crystal Oscillator Control
        (0x000 => ctrl: ReadWrite<u32, CTRL::Register>),
        /// Crystal Oscillator Status
        (0x004 => status: ReadWrite<u32, STATUS::Register>),
        /// Crystal Oscillator pause control.
        /// This is used to save power by pausing the XOSC.
        /// On power-up this field is initialised to WAKE.
        /// An invalid write will also select WAKE.
        /// WARNING: stop the PLLs before selecting dormant mode.
        /// WARNING: setup the irq before selecting dormant mode
        (0x008 => dormant: ReadWrite<u32, DORMANT::Register>),
        /// Controls the startup delay
        (0x00C => startup: ReadWrite<u32, STARTUP::Register>),
        /// A down counter running at the xosc frequency which counts to zero and stops..
        /// To start the counter write a non-zero value.
        /// Can be used for short software pauses when setting up time sensitive
        (0x010 => count: ReadWrite<u32>),
        (0x014 => @END),
    }
}

register_bitfields![u32,
CTRL [
    /// On power-up this field is initialised to DISABLE and the chip runs from the ROSC.
///                             If the chip has subsequently been programmed to run from the XOSC then setting this field to DISABLE may lock-up the chip. If  this is a concern then run the clk_ref from the ROSC and enable the clk_sys RESUS feature.
///                             The 12-bit code is intended to give some protection against accidental writes. An invalid setting will retain the previous value. The actual value being used can be read from STATUS_ENABLED
    ENABLE OFFSET(12) NUMBITS(12) [
        DISABLE = 0xd1e,
        ENABLE = 0xfab,
    ],
    /// The 12-bit code is intended to give some protection against accidental writes. An invalid setting will retain the previous value. The actual value being used can be read from STATUS_FREQ_RANGE
    FREQ_RANGE OFFSET(0) NUMBITS(12) [
        FREQ_1_15MHZ = 0xaa0,
        FREQ_10_30MHZ = 0xaa1,
        FREQ_25_60MHZ = 0xaa2,
        FREQ_40_100MHZ = 0xaa3,
    ]
],
STATUS [
    /// Oscillator is running and stable
    STABLE OFFSET(31) NUMBITS(1) [],
    /// An invalid value has been written to CTRL_ENABLE or CTRL_FREQ_RANGE or DORMANT
    BADWRITE OFFSET(24) NUMBITS(1) [],
    /// Oscillator is enabled but not necessarily running and stable, resets to 0
    ENABLED OFFSET(12) NUMBITS(1) [],
    /// The current frequency range setting
    FREQ_RANGE OFFSET(0) NUMBITS(2) [
        FREQ_1_15MHZ = 0x0,
        FREQ_10_30MHZ = 0x1,
        FREQ_25_60MHZ = 0x2,
        FREQ_40_100MHZ = 0x3,
    ]
],
DORMANT [
    /// This is used to save power by pausing the XOSC
///                             On power-up this field is initialised to WAKE
///                             An invalid write will also select WAKE
///                             Warning: stop the PLLs before selecting dormant mode
///                             Warning: setup the irq before selecting dormant mode
    VALUE OFFSET(0) NUMBITS(32) [
        DORMANT = 0x636f6d61,
        WAKE = 0x77616b65,
    ]
],
STARTUP [
    /// Multiplies the startup_delay by 4, just in case. The reset value is controlled by a mask-programmable tiecell and is provided in case we are booting from XOSC and the default startup delay is insufficient. The reset value is 0x0.
    X4 OFFSET(20) NUMBITS(1) [],
    /// in multiples of 256*xtal_period. The reset value of 0xc4 corresponds to approx 50 000 cycles.
    DELAY OFFSET(0) NUMBITS(14) []
],
COUNT [

    COUNT OFFSET(0) NUMBITS(16) []
]
];

const XOSC_BASE: StaticRef<XoscRegisters> =
    unsafe { StaticRef::new(0x40048000 as *const XoscRegisters) };

pub struct Xosc {
    registers: StaticRef<XoscRegisters>,
}

impl Xosc {
    pub const fn new() -> Self {
        Self {
            registers: XOSC_BASE,
        }
    }

    pub fn init(&self) {
        self.registers.ctrl.modify(CTRL::FREQ_RANGE::FREQ_1_15MHZ);
        // This delay is from the RP2350 manual, page 552, section 8.2.4, and from the Pico SDK
        // implementation of the XOSC driver.
        let startup_delay = (((12 * 1000000) / 1000) + 128) / 256;
        self.registers
            .startup
            .modify(STARTUP::DELAY.val(startup_delay));
        self.registers.ctrl.modify(CTRL::ENABLE::ENABLE);
        // wait for the oscillator to become stable
        while !self.registers.status.is_set(STATUS::STABLE) {}
    }

    pub fn disable(&self) {
        self.registers.ctrl.modify(CTRL::ENABLE::DISABLE);
    }

    /// disable the oscillator until an interrupt arrives
    pub fn dormant(&self) {
        self.registers.dormant.modify(DORMANT::VALUE::DORMANT);
        // wait for the oscillator to become stable
        while !self.registers.status.is_set(STATUS::STABLE) {}
    }
}
