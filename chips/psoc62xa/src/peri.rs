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
        (0xC00 => clock_ctl_0: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xC04 => clock_ctl_1: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xC08 => clock_ctl_2: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xC0C => clock_ctl_3: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xC10 => clock_ctl_4: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xC14 => clock_ctl_5: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xC18 => clock_ctl_6: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xC1C => clock_ctl_7: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xC20 => clock_ctl_8: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xC24 => clock_ctl_9: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xC28 => clock_ctl_10: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xC2C => clock_ctl_11: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xC30 => clock_ctl_12: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xC34 => clock_ctl_13: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xC38 => clock_ctl_14: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xC3C => clock_ctl_15: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xC40 => clock_ctl_16: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xC44 => clock_ctl_17: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xC48 => clock_ctl_18: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xC4C => clock_ctl_19: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xC50 => clock_ctl_20: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xC54 => clock_ctl_21: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xC58 => clock_ctl_22: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xC5C => clock_ctl_23: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xC60 => clock_ctl_24: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xC64 => clock_ctl_25: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xC68 => clock_ctl_26: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xC6C => clock_ctl_27: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xC70 => clock_ctl_28: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xC74 => clock_ctl_29: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xC78 => clock_ctl_30: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xC7C => clock_ctl_31: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xC80 => clock_ctl_32: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xC84 => clock_ctl_33: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xC88 => clock_ctl_34: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xC8C => clock_ctl_35: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xC90 => clock_ctl_36: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xC94 => clock_ctl_37: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xC98 => clock_ctl_38: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xC9C => clock_ctl_39: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xCA0 => clock_ctl_40: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xCA4 => clock_ctl_41: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xCA8 => clock_ctl_42: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xCAC => clock_ctl_43: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xCB0 => clock_ctl_44: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xCB4 => clock_ctl_45: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xCB8 => clock_ctl_46: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xCBC => clock_ctl_47: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xCC0 => clock_ctl_48: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xCC4 => clock_ctl_49: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xCC8 => clock_ctl_50: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xCCC => clock_ctl_51: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xCD0 => clock_ctl_52: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xCD4 => clock_ctl_53: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xCD8 => _reserved4),
        (0x1000 => div_8_ctl_0: ReadWrite<u32, DIV_8_CTL::Register>),
        (0x1004 => div_8_ctl_1: ReadWrite<u32, DIV_8_CTL::Register>),
        (0x1008 => div_8_ctl_2: ReadWrite<u32, DIV_8_CTL::Register>),
        (0x100C => div_8_ctl_3: ReadWrite<u32, DIV_8_CTL::Register>),
        (0x1010 => div_8_ctl_4: ReadWrite<u32, DIV_8_CTL::Register>),
        (0x1014 => div_8_ctl_5: ReadWrite<u32, DIV_8_CTL::Register>),
        (0x1018 => div_8_ctl_6: ReadWrite<u32, DIV_8_CTL::Register>),
        (0x101C => div_8_ctl_7: ReadWrite<u32, DIV_8_CTL::Register>),
        (0x1020 => _reserved5),
        (0x1400 => div_16_ctl_0: ReadWrite<u32, DIV_16_CTL::Register>),
        (0x1404 => div_16_ctl_1: ReadWrite<u32, DIV_16_CTL::Register>),
        (0x1408 => div_16_ctl_2: ReadWrite<u32, DIV_16_CTL::Register>),
        (0x140C => div_16_ctl_3: ReadWrite<u32, DIV_16_CTL::Register>),
        (0x1410 => div_16_ctl_4: ReadWrite<u32, DIV_16_CTL::Register>),
        (0x1414 => div_16_ctl_5: ReadWrite<u32, DIV_16_CTL::Register>),
        (0x1418 => div_16_ctl_6: ReadWrite<u32, DIV_16_CTL::Register>),
        (0x141C => div_16_ctl_7: ReadWrite<u32, DIV_16_CTL::Register>),
        (0x1420 => div_16_ctl_8: ReadWrite<u32, DIV_16_CTL::Register>),
        (0x1424 => div_16_ctl_9: ReadWrite<u32, DIV_16_CTL::Register>),
        (0x1428 => div_16_ctl_10: ReadWrite<u32, DIV_16_CTL::Register>),
        (0x142C => div_16_ctl_11: ReadWrite<u32, DIV_16_CTL::Register>),
        (0x1430 => div_16_ctl_12: ReadWrite<u32, DIV_16_CTL::Register>),
        (0x1434 => div_16_ctl_13: ReadWrite<u32, DIV_16_CTL::Register>),
        (0x1438 => div_16_ctl_14: ReadWrite<u32, DIV_16_CTL::Register>),
        (0x143C => div_16_ctl_15: ReadWrite<u32, DIV_16_CTL::Register>),
        (0x1440 => _reserved6),
        (0x1800 => div_16_5_ctl_0: ReadWrite<u32, DIV_16_5_CTL::Register>),
        (0x1804 => div_16_5_ctl_1: ReadWrite<u32, DIV_16_5_CTL::Register>),
        (0x1808 => div_16_5_ctl_2: ReadWrite<u32, DIV_16_5_CTL::Register>),
        (0x180C => div_16_5_ctl_3: ReadWrite<u32, DIV_16_5_CTL::Register>),
        (0x1810 => _reserved7),
        (0x1C00 => div_24_5_ctl_0: ReadWrite<u32, DIV_24_5_CTL::Register>),
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
        self.registers
            .div_16_5_ctl_3
            .modify(DIV_16_5_CTL::INT16_DIV.val(3) + DIV_16_5_CTL::FRAC5_DIV.val(20));
        self.registers.div_cmd.write(
            DIV_CMD::ENABLE::SET
                + DIV_CMD::DIV_SEL.val(3)
                + DIV_CMD::TYPE_SEL::DIV16_5
                + DIV_CMD::PA_TYPE_SEL.val(3)
                + DIV_CMD::PA_DIV_SEL.val(255),
        );

        while self.registers.div_cmd.read(DIV_CMD::ENABLE) == 1 {}

        self.registers
            .clock_ctl_5
            .modify(CLOCK_CTL::DIV_SEL.val(3) + CLOCK_CTL::TYPE_SEL::DIV16_5);
    }

    pub fn init_alarm_clock(&self) {
        self.registers
            .div_cmd
            .write(DIV_CMD::DISABLE::SET + DIV_CMD::DIV_SEL.val(0) + DIV_CMD::TYPE_SEL::DIV8_0);
        self.registers
            .div_8_ctl_0
            .modify(DIV_8_CTL::INT8_DIV.val(7));
        self.registers.div_cmd.write(
            DIV_CMD::ENABLE::SET
                + DIV_CMD::DIV_SEL.val(0)
                + DIV_CMD::TYPE_SEL::DIV8_0
                + DIV_CMD::PA_TYPE_SEL.val(3)
                + DIV_CMD::PA_DIV_SEL.val(255),
        );

        while self.registers.div_cmd.read(DIV_CMD::ENABLE) == 1 {}

        self.registers
            .clock_ctl_15
            .modify(CLOCK_CTL::DIV_SEL.val(0) + CLOCK_CTL::TYPE_SEL::DIV8_0);
    }
}
