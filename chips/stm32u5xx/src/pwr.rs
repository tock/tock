// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use kernel::utilities::registers::interfaces::ReadWriteable;
use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::utilities::StaticRef;

register_structs! {
    /// Power control
    PwrRegisters {
        (0x000 => _reserved),
        /// PWR supply voltage monitoring control register
        (0x010 => pwr_svmcr: ReadWrite<u32, PWR_SVMCR::Register>),
        (0x014 => @END),
    }
}
register_bitfields![u32,
    PWR_SVMCR [
        // This bit is used to validate the VDDA supply for electrical and logical isolation purpose.
        // Setting this bit is mandatory to use the analog peripherals.
        // If VDDA is not always present in the application, the VDDA voltage monitor can be used to determine whether this supply is ready or not.
        /// VDDA independent analog supply valid
        ASV OFFSET(30) NUMBITS(1) []
    ],
];
const PWR_BASE: StaticRef<PwrRegisters> =
    unsafe { StaticRef::new(0x46020800 as *const PwrRegisters) };

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
        self.registers.pwr_svmcr.modify(PWR_SVMCR::ASV::SET);
    }
}
