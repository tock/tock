// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025 SRL.

use kernel::utilities::registers::{
    interfaces::{ReadWriteable, Readable, Writeable},
    register_bitfields, register_structs, ReadWrite,
};
use kernel::utilities::StaticRef;

register_structs! {
    PeriRegisters {
        (0x000 => _reserved0),
        (0x200 => timeout_ctl: ReadWrite<u32>),
        (0x204 => _reserved1),
        (0x220 => tr_cmd: ReadWrite<u32, TR_CMD::Register>),
        (0x224 => _reserved2),
        (0x400 => div_cmd: ReadWrite<u32, DIV_CMD::Register>),
        (0x404 => _reserved3),
        (0xC00 => clock_ctl: [ReadWrite<u32, CLOCK_CTL::Register>; 54]),
        (0xCD8 => _reserved4),
        (0x1000 => div_8_ctl: [ReadWrite<u32, DIV_8_CTL::Register>; 8]),
        (0x1020 => _reserved5),
        (0x1400 => div_16_ctl: [ReadWrite<u32, DIV_16_CTL::Register>; 16]),
        (0x1440 => _reserved6),
        (0x1800 => div_16_5_ctl: [ReadWrite<u32, DIV_16_5_CTL::Register>; 4]),
        (0x1810 => _reserved7),
        (0x1C00 => div_24_5_ctl: ReadWrite<u32, DIV_24_5_CTL::Register>),
        (0x1C04 => _reserved8),
        (0x2000 => ecc_ctl: ReadWrite<u32, ECC_CTL::Register>),
        (0x2004 => @END),
    }
}
register_bitfields![u32,
TIMEOUT_CTL [
    TIMEOUT OFFSET(0) NUMBITS(16) []
],
TR_CMD [
    TR_SEL OFFSET(0) NUMBITS(8) [],
    GROUP_SEL OFFSET(8) NUMBITS(5) [],
    TR_EDGE OFFSET(29) NUMBITS(1) [],
    OUT_SEL OFFSET(30) NUMBITS(1) [],
    ACTIVATE OFFSET(31) NUMBITS(1) []
],
DIV_CMD [
    DIV_SEL OFFSET(0) NUMBITS(8) [],
    TYPE_SEL OFFSET(8) NUMBITS(2) [
        DIV8_0 = 0b00,
        DIV16_0 = 0b01,
        DIV16_5 = 0b10,
        DIV24_5 = 0b11,
    ],
    PA_DIV_SEL OFFSET(16) NUMBITS(8) [],
    PA_TYPE_SEL OFFSET(24) NUMBITS(2) [
        DIV8_0 = 0b00,
        DIV16_0 = 0b01,
        DIV16_5 = 0b10,
        DIV24_5 = 0b11,
    ],
    DISABLE OFFSET(30) NUMBITS(1) [],
    ENABLE OFFSET(31) NUMBITS(1) []
],
ECC_CTL [
    WORD_ADDR OFFSET(0) NUMBITS(11) [],
    ECC_EN OFFSET(16) NUMBITS(1) [],
    ECC_INJ_EN OFFSET(18) NUMBITS(1) [],
    PARITY OFFSET(24) NUMBITS(8) []
],
CLOCK_CTL [
    DIV_SEL OFFSET(0) NUMBITS(8) [],
    TYPE_SEL OFFSET(8) NUMBITS(2) [
        DIV8_0 = 0b00,
        DIV16_0 = 0b01,
        DIV16_5 = 0b10,
        DIV24_5 = 0b11,
    ]
],
DIV_8_CTL [
    EN OFFSET(0) NUMBITS(1) [],
    INT8_DIV OFFSET(8) NUMBITS(8) []
],
DIV_16_CTL [
    EN OFFSET(0) NUMBITS(1) [],
    INT16_DIV OFFSET(8) NUMBITS(16) []
],
DIV_16_5_CTL [
    EN OFFSET(0) NUMBITS(1) [],
    FRAC5_DIV OFFSET(3) NUMBITS(5) [],
    INT16_DIV OFFSET(8) NUMBITS(16) []
],
DIV_24_5_CTL [
    EN OFFSET(0) NUMBITS(1) [],
    FRAC5_DIV OFFSET(3) NUMBITS(5) [],
    INT24_DIV OFFSET(8) NUMBITS(24) []
],
];
const PERI_BASE: StaticRef<PeriRegisters> =
    unsafe { StaticRef::new(0x40000000 as *const PeriRegisters) };

pub struct Peri {
    registers: StaticRef<PeriRegisters>,
}

impl Peri {
    pub const fn new() -> Peri {
        Peri {
            registers: PERI_BASE,
        }
    }

    pub fn init_uart_clock(&self) {
        self.registers
            .div_cmd
            .write(DIV_CMD::DISABLE::SET + DIV_CMD::DIV_SEL.val(3) + DIV_CMD::TYPE_SEL::DIV16_5);
        self.registers.div_16_5_ctl[3]
            .modify(DIV_16_5_CTL::INT16_DIV.val(3) + DIV_16_5_CTL::FRAC5_DIV.val(20));
        self.registers.div_cmd.write(
            DIV_CMD::ENABLE::SET
                + DIV_CMD::DIV_SEL.val(3)
                + DIV_CMD::TYPE_SEL::DIV16_5
                + DIV_CMD::PA_TYPE_SEL.val(3)
                + DIV_CMD::PA_DIV_SEL.val(255),
        );

        while self.registers.div_cmd.read(DIV_CMD::ENABLE) == 1 {}

        self.registers.clock_ctl[5]
            .modify(CLOCK_CTL::DIV_SEL.val(3) + CLOCK_CTL::TYPE_SEL::DIV16_5);
    }

    pub fn init_alarm_clock(&self) {
        self.registers
            .div_cmd
            .write(DIV_CMD::DISABLE::SET + DIV_CMD::DIV_SEL.val(0) + DIV_CMD::TYPE_SEL::DIV8_0);
        self.registers.div_8_ctl[0].modify(DIV_8_CTL::INT8_DIV.val(7));
        self.registers.div_cmd.write(
            DIV_CMD::ENABLE::SET
                + DIV_CMD::DIV_SEL.val(0)
                + DIV_CMD::TYPE_SEL::DIV8_0
                + DIV_CMD::PA_TYPE_SEL.val(3)
                + DIV_CMD::PA_DIV_SEL.val(255),
        );

        while self.registers.div_cmd.read(DIV_CMD::ENABLE) == 1 {}

        self.registers.clock_ctl[15]
            .modify(CLOCK_CTL::DIV_SEL.val(0) + CLOCK_CTL::TYPE_SEL::DIV8_0);
    }
}
