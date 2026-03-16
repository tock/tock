// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Power control peripheral for the STM32WLE5xx series.
//! This is a partial implementation focusing on exposing the functionality
//! required for Sub-GHz radio operation.

use kernel::utilities::registers::interfaces::{ReadWriteable, Readable};
use kernel::utilities::registers::{register_bitfields, ReadWrite, WriteOnly};
use kernel::utilities::StaticRef;

const PWR: StaticRef<PwrRegisters> = unsafe { StaticRef::new(0x5800_0400 as *const _) };

#[repr(C)]
struct PwrRegisters {
    cr1: ReadWrite<u32>,
    cr2: ReadWrite<u32>,
    cr3: ReadWrite<u32>,
    cr4: ReadWrite<u32>,
    sr1: ReadWrite<u32>,
    sr2: ReadWrite<u32, SR2::Register>,
    scr: WriteOnly<u32>,
    pub cr5: ReadWrite<u32>,
    pub pucra: ReadWrite<u32>,
    pub pdcra: ReadWrite<u32>,
    pub pucrb: ReadWrite<u32>,
    pub pdcrb: ReadWrite<u32>,
    pub pucrc: ReadWrite<u32>,
    pub pdcrc: ReadWrite<u32>, // Offset 0x034
    _reserved0: [u32; 9],
    pub pucrh: ReadWrite<u32>, // Offset 0x058
    pub pdcrh: ReadWrite<u32>, // Offset 0x05C
    _reserved1: [u32; 10],
    pub extscr: ReadWrite<u32>,                             // Offset 0x088
    pub subghzspicr: ReadWrite<u32, SUBGHZSPICR::Register>, // Offset 0x090
}

register_bitfields![ u32,
   SR2 [
        PVMO3 OFFSET(14) NUMBITS(1),
        PVDO OFFSET(11) NUMBITS(1),
        VOSF OFFSET(10) NUMBITS(1),
        REGLPF OFFSET(9) NUMBITS(1),
        REGLPS OFFSET(8) NUMBITS(1),
        FLASHRDY OFFSET(7) NUMBITS(1),
        REGMRS OFFSET(6) NUMBITS(1),
        RFEOLF OFFSET(5) NUMBITS(1),
        LDORDY OFFSET(4) NUMBITS(1),
        SMPSRDY OFFSET(3) NUMBITS(1),
        RFBUSYMS OFFSET(2) NUMBITS(1),
        RFBUSYS OFFSET(1) NUMBITS(1),
    ],
    SUBGHZSPICR [
        NSS OFFSET(15) NUMBITS(1),
    ]
];

pub struct Pwr {
    registers: StaticRef<PwrRegisters>,
}

impl Pwr {
    pub fn new() -> Pwr {
        assert!(core::mem::size_of::<PwrRegisters>() == 0x94);
        Pwr { registers: PWR }
    }

    pub fn is_rfbusys(&self) -> bool {
        self.registers.sr2.is_set(SR2::RFBUSYS)
    }

    pub fn set_nss(&self) {
        self.registers.subghzspicr.modify(SUBGHZSPICR::NSS::SET);
        assert!(self.is_set_nss());
    }

    pub fn clear_nss(&self) {
        self.registers.subghzspicr.modify(SUBGHZSPICR::NSS::CLEAR);
        assert!(!self.is_set_nss());
    }

    pub fn is_set_nss(&self) -> bool {
        self.registers.subghzspicr.is_set(SUBGHZSPICR::NSS)
    }
}
