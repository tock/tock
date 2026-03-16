// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Author: Kamil Duljas <kamil.duljas@gmail.com>

use kernel::utilities::registers::interfaces::Readable;
use kernel::utilities::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::utilities::StaticRef;

register_structs! {
    /// Power control
    PwrRegisters {
        /// Power control register 1
        (0x000 => cr1: ReadWrite<u32, CR1::Register>),
        (0x004 => _reserved0),
        /// Power status register 2
        (0x014 => sr2: ReadOnly<u32, SR2::Register>),
        (0x018 => @END),
    }
}
register_bitfields![u32,
    // PWR control register 1 (CR1) — STM32L4xx
    CR1 [
        /// Backup domain write protection disable
        DBP OFFSET(8) NUMBITS(1) [],
        /// Voltage scaling range selection (bits 10:9)
        /// 00: Forbidden, 01: Range1, 10: Range2, 11: Forbidden
        VOS OFFSET(9) NUMBITS(2) [
            Range1 = 0b01,
            Range2 = 0b10
        ]
    ],

    // PWR status register 2 (SR2) — STM32L4xx
    SR2 [
        /// Voltage scaling flag (1: VOS change ongoing)
        VOSF OFFSET(10) NUMBITS(1) []
    ],
];
const PWR_BASE: StaticRef<PwrRegisters> =
    unsafe { StaticRef::new(0x40007000 as *const PwrRegisters) };

pub struct Pwr {
    registers: StaticRef<PwrRegisters>,
}

impl Pwr {
    pub fn new() -> Self {
        let pwr = Self {
            registers: PWR_BASE,
        };
        pwr
    }

    pub(crate) fn get_vos(&self) -> VOS {
        match self.registers.cr1.read(CR1::VOS) {
            1 => VOS::Range1,
            2 => VOS::Range2,
            _ => todo!(),
        }
    }

    pub(crate) fn is_vos_ready(&self) -> bool {
        match self.registers.sr2.read(SR2::VOSF) {
            0 => true,
            _ => false,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum VOS {
    Range1 = 1,
    Range2 = 2,
}
