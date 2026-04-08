// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

use enum_primitive::cast::FromPrimitive;
use enum_primitive::enum_from_primitive;
use kernel::utilities::registers::interfaces::ReadWriteable;
use kernel::utilities::registers::{register_bitfields, ReadWrite};
use kernel::utilities::StaticRef;

use crate::gpio;

/// System configuration controller
#[repr(C)]
struct SyscfgRegisters {
    /// memory remap register
    memrm: ReadWrite<u32, MEMRM::Register>,
    /// configuration register 1
    cfgr1: ReadWrite<u32, CFGR1::Register>,
    /// external interrupt configuration register 1
    exticr1: ReadWrite<u32, EXTICR1::Register>,
    /// external interrupt configuration register 2
    exticr2: ReadWrite<u32, EXTICR2::Register>,
    /// external interrupt configuration register 3
    exticr3: ReadWrite<u32, EXTICR3::Register>,
    /// external interrupt configuration register 4
    exticr4: ReadWrite<u32, EXTICR4::Register>,
    /// SRAM control and status register
    scsr: ReadWrite<u32, SCSR::Register>,
    /// configuration register 2
    cfgr2: ReadWrite<u32, CFGR2::Register>,
    /// SRAM write protection register
    swpr: ReadWrite<u32, SWPR::Register>,
    /// SRAM key register
    skr: ReadWrite<u32, SKR::Register>,
    // RESERVED (0x028 - 0x204)
    _reserved0: [u32; 120],
    // Radio debug control register
    rfdcr: ReadWrite<u32, RFDCR::Register>,
}

register_bitfields![u32,
    MEMRM [
        /// Memory mapping selection
        MEM_MODE OFFSET(0) NUMBITS(3) [],
    ],
    CFGR1 [
        /// I2C3 Fast-mode Plus driving capability activation
        I2C3_FMP OFFSET(22) NUMBITS(1) [],
        /// I2C2 Fast-mode Plus driving capability activation
        I2C2_FMP OFFSET(21) NUMBITS(1) [],
        /// I2C1 Fast-mode Plus driving capability activation
        I2C1_FMP OFFSET(20) NUMBITS(1) [],
        /// Fast-mode Plus (Fm+) driving capability activation on PB9
        I2C_PB9_FMP OFFSET(19) NUMBITS(1) [],
        /// Fast-mode Plus (Fm+) driving capability activation on PB8
        I2C_PB8_FMP OFFSET(18) NUMBITS(1) [],
        /// Fast-mode Plus (Fm+) driving capability activation on PB7
        I2C_PB7_FMP OFFSET(17) NUMBITS(1) [],
        /// Fast-mode Plus (Fm+) driving capability activation on PB6
        I2C_PB6_FMP OFFSET(16) NUMBITS(1) [],
        /// I/O analog switch voltage booster enable
        BOOSTEN OFFSET(8) NUMBITS(1) [],
    ],
    EXTICR1 [
        /// EXTI3 configuration bits
        EXTI3 OFFSET(12) NUMBITS(3) [],
        /// EXTI2 configuration bits
        EXTI2 OFFSET(8) NUMBITS(3) [],
        /// EXTI1 configuration bits
        EXTI1 OFFSET(4) NUMBITS(3) [],
        /// EXTI0 configuration bits
        EXTI0 OFFSET(0) NUMBITS(3) []
    ],
    EXTICR2 [
        /// EXTI7 configuration bits
        EXTI7 OFFSET(12) NUMBITS(3) [],
        /// EXTI6 configuration bits
        EXTI6 OFFSET(8) NUMBITS(3) [],
        /// EXTI5 configuration bits
        EXTI5 OFFSET(4) NUMBITS(3) [],
        /// EXTI4 configuration bits
        EXTI4 OFFSET(0) NUMBITS(3) []
    ],
    EXTICR3 [
        /// EXTI11 configuration bits
        EXTI11 OFFSET(12) NUMBITS(3) [],
        /// EXTI10 configuration bits
        EXTI10 OFFSET(8) NUMBITS(3) [],
        /// EXTI9 configuration bits
        EXTI9 OFFSET(4) NUMBITS(3) [],
        /// EXTI8 configuration bits
        EXTI8 OFFSET(0) NUMBITS(3) []
    ],
    EXTICR4 [
        /// EXTI15 configuration bits
        EXTI15 OFFSET(12) NUMBITS(4) [],
        /// EXTI14 configuration bits
        EXTI14 OFFSET(8) NUMBITS(4) [],
        /// EXTI13 configuration bits
        EXTI13 OFFSET(4) NUMBITS(4) [],
        /// EXTI12 configuration bits
        EXTI12 OFFSET(0) NUMBITS(4) []
    ],
    SCSR [
        /// PKA SRAM busy by erase operation
        PKASRAMBSY OFFSET(8) NUMBITS(1) [],
        /// SRAM1 or SRAM2 busy by erase operation
        SRAMBSY OFFSET(1) NUMBITS(1) [],
        /// SRAM2 erase
        SRAM2ER OFFSET(0) NUMBITS(1) []
    ],
    CFGR2 [
        /// SRAM2 parity error flag
        SPF OFFSET(8) NUMBITS(1) [],
        /// ECC Lock
        ECCL OFFSET(3) NUMBITS(1) [],
        /// PVD lock enable bit
        PVDL OFFSET(2) NUMBITS(1) [],
        ///SRAM2 parity lock bit
        SPL OFFSET(1) NUMBITS(1) [],
        /// CPU Lockup (hardfault) output enable bit
        CLL OFFSET(0) NUMBITS(1) []
    ],
    SWPR [
        /// SRAM2 1 Kbyte page 31 write protection
        P31 OFFSET(31) NUMBITS(1) [],
        /// SRAM2 1 Kbyte page 30 write protection
        P30 OFFSET(30) NUMBITS(1) [],
        /// SRAM2 1 Kbyte page 29 write protection
        P29 OFFSET(29) NUMBITS(1) [],
        /// SRAM2 1 Kbyte page 28 write protection
        P28 OFFSET(28) NUMBITS(1) [],
        /// SRAM2 1 Kbyte page 27 write protection
        P27 OFFSET(27) NUMBITS(1) [],
        /// SRAM2 1 Kbyte page 26 write protection
        P26 OFFSET(26) NUMBITS(1) [],
        /// SRAM2 1 Kbyte page 25 write protection
        P25 OFFSET(25) NUMBITS(1) [],
        /// SRAM2 1 Kbyte page 24 write protection
        P24 OFFSET(24) NUMBITS(1) [],
        /// SRAM2 1 Kbyte page 23 write protection
        P23 OFFSET(23) NUMBITS(1) [],
        /// SRAM2 1 Kbyte page 22 write protection
        P22 OFFSET(22) NUMBITS(1) [],
        /// SRAM2 1 Kbyte page 21 write protection
        P21 OFFSET(21) NUMBITS(1) [],
        /// SRAM2 1 Kbyte page 20 write protection
        P20 OFFSET(20) NUMBITS(1) [],
        /// SRAM2 1 Kbyte page 19 write protection
        P19 OFFSET(19) NUMBITS(1) [],
        /// SRAM2 1 Kbyte page 18 write protection
        P18 OFFSET(18) NUMBITS(1) [],
        /// SRAM2 1 Kbyte page 17 write protection
        P17 OFFSET(17) NUMBITS(1) [],
        /// SRAM2 1 Kbyte page 16 write protection
        P16 OFFSET(16) NUMBITS(1) [],
        /// SRAM2 1 Kbyte page 15 write protection
        P15 OFFSET(15) NUMBITS(1) [],
        /// SRAM2 1 Kbyte page 14 write protection
        P14 OFFSET(14) NUMBITS(1) [],
        /// SRAM2 1 Kbyte page 13 write protection
        P13 OFFSET(13) NUMBITS(1) [],
        /// SRAM2 1 Kbyte page 12 write protection
        P12 OFFSET(12) NUMBITS(1) [],
        /// SRAM2 1 Kbyte page 11 write protection
        P11 OFFSET(11) NUMBITS(1) [],
        /// SRAM2 1 Kbyte page 10 write protection
        P10 OFFSET(10) NUMBITS(1) [],
        /// SRAM2 1 Kbyte page 9 write protection
        P9 OFFSET(9) NUMBITS(1) [],
        /// SRAM2 1 Kbyte page 8 write protection
        P8 OFFSET(8) NUMBITS(1) [],
        /// SRAM2 1 Kbyte page 7 write protection
        P7 OFFSET(7) NUMBITS(1) [],
        /// SRAM2 1 Kbyte page 6 write protection
        P6 OFFSET(6) NUMBITS(1) [],
        /// SRAM2 1 Kbyte page 5 write protection
        P5 OFFSET(5) NUMBITS(1) [],
        /// SRAM2 1 Kbyte page 4 write protection
        P4 OFFSET(4) NUMBITS(1) [],
        /// SRAM2 1 Kbyte page 3 write protection
        P3 OFFSET(3) NUMBITS(1) [],
        /// SRAM2 1 Kbyte page 2 write protection
        P2 OFFSET(2) NUMBITS(1) [],
        /// SRAM2 1 Kbyte page 1 write protection
        P1 OFFSET(1) NUMBITS(1) [],
        /// SRAM2 1 Kbyte page 0 write protection
        P0 OFFSET(0) NUMBITS(1) [],
    ],
    SKR [
        /// SRAM2 write protection key for software erase
        ///
        /// The following steps are required to unlock the
        /// write protection of the SRAM2ER bit in the
        /// SYSCFG_SCSR register.
        ///   1. Write 0xCA into Key[7:0].
        ///   2. Write 0x53 into Key[7:0].
        /// Writing a wrong key reactivates the write protection.
        KEY OFFSET(0) NUMBITS(8) []
    ],
    RFDCR [
        /// Radio debug test bus selection
        RFTBSEL OFFSET(0) NUMBITS(1) []
    ]
];

const SYSCFG_BASE: StaticRef<SyscfgRegisters> =
    unsafe { StaticRef::new(0x40010000 as *const SyscfgRegisters) };

enum_from_primitive! {
    #[repr(u32)]
    /// SYSCFG EXTI configuration [^1]
    ///
    /// [^1]: Section 8.2.2, page 197 of reference manual
    enum ExtiCrId {
        PA = 0b0000,
        PB = 0b0001,
        PC = 0b0010,
        PD = 0b0011,
        PE = 0b0100,
        PF = 0b0101,
        PG = 0b0110,
        PH = 0b0111,
    }
}

pub struct Syscfg {
    registers: StaticRef<SyscfgRegisters>,
}

impl Syscfg {
    pub const fn new() -> Self {
        assert!(size_of::<SyscfgRegisters>() == 0x20C);
        Self {
            registers: SYSCFG_BASE,
        }
    }

    /// Configures the SYSCFG_EXTICR{1, 2, 3, 4} registers
    pub fn configure_interrupt(&self, pinid: gpio::PinId) {
        let exticrid = self.get_exticrid_from_port_num(pinid.get_port_number());

        let pin_num = pinid.get_pin_number();
        match pin_num {
            // SYSCFG_EXTICR1
            0b0000 => self
                .registers
                .exticr1
                .modify(EXTICR1::EXTI0.val(exticrid as u32)),
            0b0001 => self
                .registers
                .exticr1
                .modify(EXTICR1::EXTI1.val(exticrid as u32)),
            0b0010 => self
                .registers
                .exticr1
                .modify(EXTICR1::EXTI2.val(exticrid as u32)),
            0b0011 => self
                .registers
                .exticr1
                .modify(EXTICR1::EXTI3.val(exticrid as u32)),
            // SYSCFG_EXTICR2
            0b0100 => self
                .registers
                .exticr2
                .modify(EXTICR2::EXTI4.val(exticrid as u32)),
            0b0101 => self
                .registers
                .exticr2
                .modify(EXTICR2::EXTI5.val(exticrid as u32)),
            0b0110 => self
                .registers
                .exticr2
                .modify(EXTICR2::EXTI6.val(exticrid as u32)),
            0b0111 => self
                .registers
                .exticr2
                .modify(EXTICR2::EXTI7.val(exticrid as u32)),
            // SYSCFG_EXTICR3
            0b1000 => self
                .registers
                .exticr3
                .modify(EXTICR3::EXTI8.val(exticrid as u32)),
            0b1001 => self
                .registers
                .exticr3
                .modify(EXTICR3::EXTI9.val(exticrid as u32)),
            0b1010 => self
                .registers
                .exticr3
                .modify(EXTICR3::EXTI10.val(exticrid as u32)),
            0b1011 => self
                .registers
                .exticr3
                .modify(EXTICR3::EXTI11.val(exticrid as u32)),
            // SYSCFG_EXTICR4
            0b1100 => self
                .registers
                .exticr4
                .modify(EXTICR4::EXTI12.val(exticrid as u32)),
            0b1101 => self
                .registers
                .exticr4
                .modify(EXTICR4::EXTI13.val(exticrid as u32)),
            0b1110 => self
                .registers
                .exticr4
                .modify(EXTICR4::EXTI14.val(exticrid as u32)),
            0b1111 => self
                .registers
                .exticr4
                .modify(EXTICR4::EXTI15.val(exticrid as u32)),
            _ => {}
        }
    }

    fn get_exticrid_from_port_num(&self, port_num: u8) -> ExtiCrId {
        ExtiCrId::from_u32(u32::from(port_num)).unwrap()
    }
}
