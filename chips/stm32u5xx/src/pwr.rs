// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use kernel::utilities::StaticRef;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable};
use kernel::utilities::registers::{ReadWrite, register_bitfields, register_structs};

register_structs! {
    /// Power control
    PwrRegisters {
        (0x000 => _reserved1),
        /// PWR voltage scaling register
        (0x00C => pwr_vosr: ReadWrite<u32, PWR_VOSR::Register>),
        /// PWR supply voltage monitoring control register
        (0x010 => pwr_svmcr: ReadWrite<u32, PWR_SVMCR::Register>),
        (0x014 => _reserved2),
        /// PWR supply voltage monitoring status registe
        (0x03C => pwr_svmsr: ReadWrite<u32, PWR_SVMSR::Register>),
        (0x040 => @END),
    }
}
register_bitfields![u32,
    PWR_VOSR [
        BOOSTEN OFFSET(18) NUMBITS(1) [],
        VOS OFFSET(16) NUMBITS(2) [
            RANGE4 = 0,
            RANGE3 = 1,
            RANGE2 = 2,
            RANGE1 = 3,
        ],
        VOSRDY OFFSET(15) NUMBITS(1) [],
        BOOSTRDY OFFSET(14) NUMBITS(1) []
    ],
    PWR_SVMCR [
        // This bit is used to validate the VDDA supply for electrical and logical isolation purpose.
        // Setting this bit is mandatory to use the analog peripherals.
        // If VDDA is not always present in the application, the VDDA voltage monitor can be used to determine whether this supply is ready or not.
        /// VDDA independent analog supply valid
        ASV OFFSET(30) NUMBITS(1) [],
        // This bit is used to validate the VDDIO2 supply for electrical and logical isolation purpose.
        // Setting this bit is mandatory to use PG[15:2]. If VDDIO2 is not always present in the application, the VDDIO2 voltage monitor can
        // be used to determine whether this supply is ready or not.
        /// VDDIO2 independent I/Os supply valid
        IO2SV OFFSET(29) NUMBITS(1) [],
        // This bit is used to validate the VDDUSB supply for electrical and logical isolation purpose.
        // Setting this bit is mandatory to use the USB/OTG_FS/OTG_HS.
        // If VDDUSB is not always present in the application, the VDDUSB voltage monitor can be used to determine whether this supply is ready or not.
        /// VDDUSB independent USB supply valid
        USV OFFSET(28) NUMBITS(1) [],
        /// VDDA independent analog supply voltage monitor 1 enable (1.6 V threshold)
        AVM1EN OFFSET(26) NUMBITS(1) []
    ],
    PWR_SVMSR [
        /// VDDA ready versus 1.6V voltage monitor
        VDDA1RDY OFFSET(26) NUMBITS(1) []
    ],
];
const PWR_BASE: StaticRef<PwrRegisters> =
    unsafe { StaticRef::new(0x46020800 as *const PwrRegisters) };

#[derive(Clone, Copy)]
pub enum VoltageScale {
    /// Range 4 (lowest power)
    Range4,
    /// Range 3
    Range3,
    /// Range 2
    Range2,
    /// Range 1 (highest frequency). This value cannot be written when VCOREMEN = 1 in TAMP_OR register.
    Range1,
}

pub struct Pwr {
    registers: StaticRef<PwrRegisters>,
}

impl Pwr {
    pub const fn new() -> Self {
        Self {
            registers: PWR_BASE,
        }
    }

    pub fn validate_vdda(&self) {
        self.registers.pwr_svmcr.modify(PWR_SVMCR::AVM1EN::SET);
        while !self.registers.pwr_svmsr.is_set(PWR_SVMSR::VDDA1RDY) {}
        self.registers.pwr_svmcr.modify(PWR_SVMCR::ASV::SET);
    }

    pub fn validate_vddio2(&self) {
        self.registers.pwr_svmcr.modify(PWR_SVMCR::IO2SV::SET);
    }

    pub fn validate_vddusb(&self) {
        self.registers.pwr_svmcr.modify(PWR_SVMCR::USV::SET);
    }

    pub fn set_voltage_scaling(&self, range: VoltageScale) {
        self.registers.pwr_vosr.modify(match range {
            VoltageScale::Range1 => PWR_VOSR::VOS::RANGE1,
            VoltageScale::Range2 => PWR_VOSR::VOS::RANGE2,
            VoltageScale::Range3 => PWR_VOSR::VOS::RANGE3,
            VoltageScale::Range4 => PWR_VOSR::VOS::RANGE4,
        });

        while !self.registers.pwr_vosr.is_set(PWR_VOSR::VOSRDY) {}
    }

    pub fn enable_epod_booster(&self) {
        self.registers.pwr_vosr.modify(PWR_VOSR::BOOSTEN::SET);
        while !self.registers.pwr_vosr.is_set(PWR_VOSR::BOOSTRDY) {}
    }
}
