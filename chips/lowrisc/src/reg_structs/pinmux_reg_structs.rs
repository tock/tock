// Generated register struct for PINMUX

// Copyright information found in source file:
// Copyright lowRISC contributors.

// Licensing information found in source file:
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

#[allow(unused_imports)]
use kernel::utilities::registers::{
    register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
};

register_structs! {
    pub PinmuxRegisters {
        (0x0 => alert_test: WriteOnly<u32, ALERT_TEST::Register>),
        (0x4 => mio_periph_insel_regwen_0: ReadWrite<u32, MIO_PERIPH_INSEL_REGWEN_0::Register>),
        (0x8 => mio_periph_insel_regwen_1: ReadWrite<u32, MIO_PERIPH_INSEL_REGWEN_1::Register>),
        (0xc => mio_periph_insel_regwen_2: ReadWrite<u32, MIO_PERIPH_INSEL_REGWEN_2::Register>),
        (0x10 => mio_periph_insel_regwen_3: ReadWrite<u32, MIO_PERIPH_INSEL_REGWEN_3::Register>),
        (0x14 => mio_periph_insel_regwen_4: ReadWrite<u32, MIO_PERIPH_INSEL_REGWEN_4::Register>),
        (0x18 => mio_periph_insel_regwen_5: ReadWrite<u32, MIO_PERIPH_INSEL_REGWEN_5::Register>),
        (0x1c => mio_periph_insel_regwen_6: ReadWrite<u32, MIO_PERIPH_INSEL_REGWEN_6::Register>),
        (0x20 => mio_periph_insel_regwen_7: ReadWrite<u32, MIO_PERIPH_INSEL_REGWEN_7::Register>),
        (0x24 => mio_periph_insel_regwen_8: ReadWrite<u32, MIO_PERIPH_INSEL_REGWEN_8::Register>),
        (0x28 => mio_periph_insel_regwen_9: ReadWrite<u32, MIO_PERIPH_INSEL_REGWEN_9::Register>),
        (0x2c => mio_periph_insel_regwen_10: ReadWrite<u32, MIO_PERIPH_INSEL_REGWEN_10::Register>),
        (0x30 => mio_periph_insel_regwen_11: ReadWrite<u32, MIO_PERIPH_INSEL_REGWEN_11::Register>),
        (0x34 => mio_periph_insel_regwen_12: ReadWrite<u32, MIO_PERIPH_INSEL_REGWEN_12::Register>),
        (0x38 => mio_periph_insel_regwen_13: ReadWrite<u32, MIO_PERIPH_INSEL_REGWEN_13::Register>),
        (0x3c => mio_periph_insel_regwen_14: ReadWrite<u32, MIO_PERIPH_INSEL_REGWEN_14::Register>),
        (0x40 => mio_periph_insel_regwen_15: ReadWrite<u32, MIO_PERIPH_INSEL_REGWEN_15::Register>),
        (0x44 => mio_periph_insel_regwen_16: ReadWrite<u32, MIO_PERIPH_INSEL_REGWEN_16::Register>),
        (0x48 => mio_periph_insel_regwen_17: ReadWrite<u32, MIO_PERIPH_INSEL_REGWEN_17::Register>),
        (0x4c => mio_periph_insel_regwen_18: ReadWrite<u32, MIO_PERIPH_INSEL_REGWEN_18::Register>),
        (0x50 => mio_periph_insel_regwen_19: ReadWrite<u32, MIO_PERIPH_INSEL_REGWEN_19::Register>),
        (0x54 => mio_periph_insel_regwen_20: ReadWrite<u32, MIO_PERIPH_INSEL_REGWEN_20::Register>),
        (0x58 => mio_periph_insel_regwen_21: ReadWrite<u32, MIO_PERIPH_INSEL_REGWEN_21::Register>),
        (0x5c => mio_periph_insel_regwen_22: ReadWrite<u32, MIO_PERIPH_INSEL_REGWEN_22::Register>),
        (0x60 => mio_periph_insel_regwen_23: ReadWrite<u32, MIO_PERIPH_INSEL_REGWEN_23::Register>),
        (0x64 => mio_periph_insel_regwen_24: ReadWrite<u32, MIO_PERIPH_INSEL_REGWEN_24::Register>),
        (0x68 => mio_periph_insel_regwen_25: ReadWrite<u32, MIO_PERIPH_INSEL_REGWEN_25::Register>),
        (0x6c => mio_periph_insel_regwen_26: ReadWrite<u32, MIO_PERIPH_INSEL_REGWEN_26::Register>),
        (0x70 => mio_periph_insel_regwen_27: ReadWrite<u32, MIO_PERIPH_INSEL_REGWEN_27::Register>),
        (0x74 => mio_periph_insel_regwen_28: ReadWrite<u32, MIO_PERIPH_INSEL_REGWEN_28::Register>),
        (0x78 => mio_periph_insel_regwen_29: ReadWrite<u32, MIO_PERIPH_INSEL_REGWEN_29::Register>),
        (0x7c => mio_periph_insel_regwen_30: ReadWrite<u32, MIO_PERIPH_INSEL_REGWEN_30::Register>),
        (0x80 => mio_periph_insel_regwen_31: ReadWrite<u32, MIO_PERIPH_INSEL_REGWEN_31::Register>),
        (0x84 => mio_periph_insel_regwen_32: ReadWrite<u32, MIO_PERIPH_INSEL_REGWEN_32::Register>),
        (0x88 => mio_periph_insel_0: ReadWrite<u32, MIO_PERIPH_INSEL_0::Register>),
        (0x8c => mio_periph_insel_1: ReadWrite<u32, MIO_PERIPH_INSEL_1::Register>),
        (0x90 => mio_periph_insel_2: ReadWrite<u32, MIO_PERIPH_INSEL_2::Register>),
        (0x94 => mio_periph_insel_3: ReadWrite<u32, MIO_PERIPH_INSEL_3::Register>),
        (0x98 => mio_periph_insel_4: ReadWrite<u32, MIO_PERIPH_INSEL_4::Register>),
        (0x9c => mio_periph_insel_5: ReadWrite<u32, MIO_PERIPH_INSEL_5::Register>),
        (0xa0 => mio_periph_insel_6: ReadWrite<u32, MIO_PERIPH_INSEL_6::Register>),
        (0xa4 => mio_periph_insel_7: ReadWrite<u32, MIO_PERIPH_INSEL_7::Register>),
        (0xa8 => mio_periph_insel_8: ReadWrite<u32, MIO_PERIPH_INSEL_8::Register>),
        (0xac => mio_periph_insel_9: ReadWrite<u32, MIO_PERIPH_INSEL_9::Register>),
        (0xb0 => mio_periph_insel_10: ReadWrite<u32, MIO_PERIPH_INSEL_10::Register>),
        (0xb4 => mio_periph_insel_11: ReadWrite<u32, MIO_PERIPH_INSEL_11::Register>),
        (0xb8 => mio_periph_insel_12: ReadWrite<u32, MIO_PERIPH_INSEL_12::Register>),
        (0xbc => mio_periph_insel_13: ReadWrite<u32, MIO_PERIPH_INSEL_13::Register>),
        (0xc0 => mio_periph_insel_14: ReadWrite<u32, MIO_PERIPH_INSEL_14::Register>),
        (0xc4 => mio_periph_insel_15: ReadWrite<u32, MIO_PERIPH_INSEL_15::Register>),
        (0xc8 => mio_periph_insel_16: ReadWrite<u32, MIO_PERIPH_INSEL_16::Register>),
        (0xcc => mio_periph_insel_17: ReadWrite<u32, MIO_PERIPH_INSEL_17::Register>),
        (0xd0 => mio_periph_insel_18: ReadWrite<u32, MIO_PERIPH_INSEL_18::Register>),
        (0xd4 => mio_periph_insel_19: ReadWrite<u32, MIO_PERIPH_INSEL_19::Register>),
        (0xd8 => mio_periph_insel_20: ReadWrite<u32, MIO_PERIPH_INSEL_20::Register>),
        (0xdc => mio_periph_insel_21: ReadWrite<u32, MIO_PERIPH_INSEL_21::Register>),
        (0xe0 => mio_periph_insel_22: ReadWrite<u32, MIO_PERIPH_INSEL_22::Register>),
        (0xe4 => mio_periph_insel_23: ReadWrite<u32, MIO_PERIPH_INSEL_23::Register>),
        (0xe8 => mio_periph_insel_24: ReadWrite<u32, MIO_PERIPH_INSEL_24::Register>),
        (0xec => mio_periph_insel_25: ReadWrite<u32, MIO_PERIPH_INSEL_25::Register>),
        (0xf0 => mio_periph_insel_26: ReadWrite<u32, MIO_PERIPH_INSEL_26::Register>),
        (0xf4 => mio_periph_insel_27: ReadWrite<u32, MIO_PERIPH_INSEL_27::Register>),
        (0xf8 => mio_periph_insel_28: ReadWrite<u32, MIO_PERIPH_INSEL_28::Register>),
        (0xfc => mio_periph_insel_29: ReadWrite<u32, MIO_PERIPH_INSEL_29::Register>),
        (0x100 => mio_periph_insel_30: ReadWrite<u32, MIO_PERIPH_INSEL_30::Register>),
        (0x104 => mio_periph_insel_31: ReadWrite<u32, MIO_PERIPH_INSEL_31::Register>),
        (0x108 => mio_periph_insel_32: ReadWrite<u32, MIO_PERIPH_INSEL_32::Register>),
        (0x10c => mio_outsel_regwen_0: ReadWrite<u32, MIO_OUTSEL_REGWEN_0::Register>),
        (0x110 => mio_outsel_regwen_1: ReadWrite<u32, MIO_OUTSEL_REGWEN_1::Register>),
        (0x114 => mio_outsel_regwen_2: ReadWrite<u32, MIO_OUTSEL_REGWEN_2::Register>),
        (0x118 => mio_outsel_regwen_3: ReadWrite<u32, MIO_OUTSEL_REGWEN_3::Register>),
        (0x11c => mio_outsel_regwen_4: ReadWrite<u32, MIO_OUTSEL_REGWEN_4::Register>),
        (0x120 => mio_outsel_regwen_5: ReadWrite<u32, MIO_OUTSEL_REGWEN_5::Register>),
        (0x124 => mio_outsel_regwen_6: ReadWrite<u32, MIO_OUTSEL_REGWEN_6::Register>),
        (0x128 => mio_outsel_regwen_7: ReadWrite<u32, MIO_OUTSEL_REGWEN_7::Register>),
        (0x12c => mio_outsel_regwen_8: ReadWrite<u32, MIO_OUTSEL_REGWEN_8::Register>),
        (0x130 => mio_outsel_regwen_9: ReadWrite<u32, MIO_OUTSEL_REGWEN_9::Register>),
        (0x134 => mio_outsel_regwen_10: ReadWrite<u32, MIO_OUTSEL_REGWEN_10::Register>),
        (0x138 => mio_outsel_regwen_11: ReadWrite<u32, MIO_OUTSEL_REGWEN_11::Register>),
        (0x13c => mio_outsel_regwen_12: ReadWrite<u32, MIO_OUTSEL_REGWEN_12::Register>),
        (0x140 => mio_outsel_regwen_13: ReadWrite<u32, MIO_OUTSEL_REGWEN_13::Register>),
        (0x144 => mio_outsel_regwen_14: ReadWrite<u32, MIO_OUTSEL_REGWEN_14::Register>),
        (0x148 => mio_outsel_regwen_15: ReadWrite<u32, MIO_OUTSEL_REGWEN_15::Register>),
        (0x14c => mio_outsel_regwen_16: ReadWrite<u32, MIO_OUTSEL_REGWEN_16::Register>),
        (0x150 => mio_outsel_regwen_17: ReadWrite<u32, MIO_OUTSEL_REGWEN_17::Register>),
        (0x154 => mio_outsel_regwen_18: ReadWrite<u32, MIO_OUTSEL_REGWEN_18::Register>),
        (0x158 => mio_outsel_regwen_19: ReadWrite<u32, MIO_OUTSEL_REGWEN_19::Register>),
        (0x15c => mio_outsel_regwen_20: ReadWrite<u32, MIO_OUTSEL_REGWEN_20::Register>),
        (0x160 => mio_outsel_regwen_21: ReadWrite<u32, MIO_OUTSEL_REGWEN_21::Register>),
        (0x164 => mio_outsel_regwen_22: ReadWrite<u32, MIO_OUTSEL_REGWEN_22::Register>),
        (0x168 => mio_outsel_regwen_23: ReadWrite<u32, MIO_OUTSEL_REGWEN_23::Register>),
        (0x16c => mio_outsel_regwen_24: ReadWrite<u32, MIO_OUTSEL_REGWEN_24::Register>),
        (0x170 => mio_outsel_regwen_25: ReadWrite<u32, MIO_OUTSEL_REGWEN_25::Register>),
        (0x174 => mio_outsel_regwen_26: ReadWrite<u32, MIO_OUTSEL_REGWEN_26::Register>),
        (0x178 => mio_outsel_regwen_27: ReadWrite<u32, MIO_OUTSEL_REGWEN_27::Register>),
        (0x17c => mio_outsel_regwen_28: ReadWrite<u32, MIO_OUTSEL_REGWEN_28::Register>),
        (0x180 => mio_outsel_regwen_29: ReadWrite<u32, MIO_OUTSEL_REGWEN_29::Register>),
        (0x184 => mio_outsel_regwen_30: ReadWrite<u32, MIO_OUTSEL_REGWEN_30::Register>),
        (0x188 => mio_outsel_regwen_31: ReadWrite<u32, MIO_OUTSEL_REGWEN_31::Register>),
        (0x18c => mio_outsel_0: ReadWrite<u32, MIO_OUTSEL_0::Register>),
        (0x190 => mio_outsel_1: ReadWrite<u32, MIO_OUTSEL_1::Register>),
        (0x194 => mio_outsel_2: ReadWrite<u32, MIO_OUTSEL_2::Register>),
        (0x198 => mio_outsel_3: ReadWrite<u32, MIO_OUTSEL_3::Register>),
        (0x19c => mio_outsel_4: ReadWrite<u32, MIO_OUTSEL_4::Register>),
        (0x1a0 => mio_outsel_5: ReadWrite<u32, MIO_OUTSEL_5::Register>),
        (0x1a4 => mio_outsel_6: ReadWrite<u32, MIO_OUTSEL_6::Register>),
        (0x1a8 => mio_outsel_7: ReadWrite<u32, MIO_OUTSEL_7::Register>),
        (0x1ac => mio_outsel_8: ReadWrite<u32, MIO_OUTSEL_8::Register>),
        (0x1b0 => mio_outsel_9: ReadWrite<u32, MIO_OUTSEL_9::Register>),
        (0x1b4 => mio_outsel_10: ReadWrite<u32, MIO_OUTSEL_10::Register>),
        (0x1b8 => mio_outsel_11: ReadWrite<u32, MIO_OUTSEL_11::Register>),
        (0x1bc => mio_outsel_12: ReadWrite<u32, MIO_OUTSEL_12::Register>),
        (0x1c0 => mio_outsel_13: ReadWrite<u32, MIO_OUTSEL_13::Register>),
        (0x1c4 => mio_outsel_14: ReadWrite<u32, MIO_OUTSEL_14::Register>),
        (0x1c8 => mio_outsel_15: ReadWrite<u32, MIO_OUTSEL_15::Register>),
        (0x1cc => mio_outsel_16: ReadWrite<u32, MIO_OUTSEL_16::Register>),
        (0x1d0 => mio_outsel_17: ReadWrite<u32, MIO_OUTSEL_17::Register>),
        (0x1d4 => mio_outsel_18: ReadWrite<u32, MIO_OUTSEL_18::Register>),
        (0x1d8 => mio_outsel_19: ReadWrite<u32, MIO_OUTSEL_19::Register>),
        (0x1dc => mio_outsel_20: ReadWrite<u32, MIO_OUTSEL_20::Register>),
        (0x1e0 => mio_outsel_21: ReadWrite<u32, MIO_OUTSEL_21::Register>),
        (0x1e4 => mio_outsel_22: ReadWrite<u32, MIO_OUTSEL_22::Register>),
        (0x1e8 => mio_outsel_23: ReadWrite<u32, MIO_OUTSEL_23::Register>),
        (0x1ec => mio_outsel_24: ReadWrite<u32, MIO_OUTSEL_24::Register>),
        (0x1f0 => mio_outsel_25: ReadWrite<u32, MIO_OUTSEL_25::Register>),
        (0x1f4 => mio_outsel_26: ReadWrite<u32, MIO_OUTSEL_26::Register>),
        (0x1f8 => mio_outsel_27: ReadWrite<u32, MIO_OUTSEL_27::Register>),
        (0x1fc => mio_outsel_28: ReadWrite<u32, MIO_OUTSEL_28::Register>),
        (0x200 => mio_outsel_29: ReadWrite<u32, MIO_OUTSEL_29::Register>),
        (0x204 => mio_outsel_30: ReadWrite<u32, MIO_OUTSEL_30::Register>),
        (0x208 => mio_outsel_31: ReadWrite<u32, MIO_OUTSEL_31::Register>),
        (0x20c => mio_pad_attr_regwen_0: ReadWrite<u32, MIO_PAD_ATTR_REGWEN_0::Register>),
        (0x210 => mio_pad_attr_regwen_1: ReadWrite<u32, MIO_PAD_ATTR_REGWEN_1::Register>),
        (0x214 => mio_pad_attr_regwen_2: ReadWrite<u32, MIO_PAD_ATTR_REGWEN_2::Register>),
        (0x218 => mio_pad_attr_regwen_3: ReadWrite<u32, MIO_PAD_ATTR_REGWEN_3::Register>),
        (0x21c => mio_pad_attr_regwen_4: ReadWrite<u32, MIO_PAD_ATTR_REGWEN_4::Register>),
        (0x220 => mio_pad_attr_regwen_5: ReadWrite<u32, MIO_PAD_ATTR_REGWEN_5::Register>),
        (0x224 => mio_pad_attr_regwen_6: ReadWrite<u32, MIO_PAD_ATTR_REGWEN_6::Register>),
        (0x228 => mio_pad_attr_regwen_7: ReadWrite<u32, MIO_PAD_ATTR_REGWEN_7::Register>),
        (0x22c => mio_pad_attr_regwen_8: ReadWrite<u32, MIO_PAD_ATTR_REGWEN_8::Register>),
        (0x230 => mio_pad_attr_regwen_9: ReadWrite<u32, MIO_PAD_ATTR_REGWEN_9::Register>),
        (0x234 => mio_pad_attr_regwen_10: ReadWrite<u32, MIO_PAD_ATTR_REGWEN_10::Register>),
        (0x238 => mio_pad_attr_regwen_11: ReadWrite<u32, MIO_PAD_ATTR_REGWEN_11::Register>),
        (0x23c => mio_pad_attr_regwen_12: ReadWrite<u32, MIO_PAD_ATTR_REGWEN_12::Register>),
        (0x240 => mio_pad_attr_regwen_13: ReadWrite<u32, MIO_PAD_ATTR_REGWEN_13::Register>),
        (0x244 => mio_pad_attr_regwen_14: ReadWrite<u32, MIO_PAD_ATTR_REGWEN_14::Register>),
        (0x248 => mio_pad_attr_regwen_15: ReadWrite<u32, MIO_PAD_ATTR_REGWEN_15::Register>),
        (0x24c => mio_pad_attr_regwen_16: ReadWrite<u32, MIO_PAD_ATTR_REGWEN_16::Register>),
        (0x250 => mio_pad_attr_regwen_17: ReadWrite<u32, MIO_PAD_ATTR_REGWEN_17::Register>),
        (0x254 => mio_pad_attr_regwen_18: ReadWrite<u32, MIO_PAD_ATTR_REGWEN_18::Register>),
        (0x258 => mio_pad_attr_regwen_19: ReadWrite<u32, MIO_PAD_ATTR_REGWEN_19::Register>),
        (0x25c => mio_pad_attr_regwen_20: ReadWrite<u32, MIO_PAD_ATTR_REGWEN_20::Register>),
        (0x260 => mio_pad_attr_regwen_21: ReadWrite<u32, MIO_PAD_ATTR_REGWEN_21::Register>),
        (0x264 => mio_pad_attr_regwen_22: ReadWrite<u32, MIO_PAD_ATTR_REGWEN_22::Register>),
        (0x268 => mio_pad_attr_regwen_23: ReadWrite<u32, MIO_PAD_ATTR_REGWEN_23::Register>),
        (0x26c => mio_pad_attr_regwen_24: ReadWrite<u32, MIO_PAD_ATTR_REGWEN_24::Register>),
        (0x270 => mio_pad_attr_regwen_25: ReadWrite<u32, MIO_PAD_ATTR_REGWEN_25::Register>),
        (0x274 => mio_pad_attr_regwen_26: ReadWrite<u32, MIO_PAD_ATTR_REGWEN_26::Register>),
        (0x278 => mio_pad_attr_regwen_27: ReadWrite<u32, MIO_PAD_ATTR_REGWEN_27::Register>),
        (0x27c => mio_pad_attr_regwen_28: ReadWrite<u32, MIO_PAD_ATTR_REGWEN_28::Register>),
        (0x280 => mio_pad_attr_regwen_29: ReadWrite<u32, MIO_PAD_ATTR_REGWEN_29::Register>),
        (0x284 => mio_pad_attr_regwen_30: ReadWrite<u32, MIO_PAD_ATTR_REGWEN_30::Register>),
        (0x288 => mio_pad_attr_regwen_31: ReadWrite<u32, MIO_PAD_ATTR_REGWEN_31::Register>),
        (0x28c => mio_pad_attr_0: ReadWrite<u32, MIO_PAD_ATTR_0::Register>),
        (0x290 => mio_pad_attr_1: ReadWrite<u32, MIO_PAD_ATTR_1::Register>),
        (0x294 => mio_pad_attr_2: ReadWrite<u32, MIO_PAD_ATTR_2::Register>),
        (0x298 => mio_pad_attr_3: ReadWrite<u32, MIO_PAD_ATTR_3::Register>),
        (0x29c => mio_pad_attr_4: ReadWrite<u32, MIO_PAD_ATTR_4::Register>),
        (0x2a0 => mio_pad_attr_5: ReadWrite<u32, MIO_PAD_ATTR_5::Register>),
        (0x2a4 => mio_pad_attr_6: ReadWrite<u32, MIO_PAD_ATTR_6::Register>),
        (0x2a8 => mio_pad_attr_7: ReadWrite<u32, MIO_PAD_ATTR_7::Register>),
        (0x2ac => mio_pad_attr_8: ReadWrite<u32, MIO_PAD_ATTR_8::Register>),
        (0x2b0 => mio_pad_attr_9: ReadWrite<u32, MIO_PAD_ATTR_9::Register>),
        (0x2b4 => mio_pad_attr_10: ReadWrite<u32, MIO_PAD_ATTR_10::Register>),
        (0x2b8 => mio_pad_attr_11: ReadWrite<u32, MIO_PAD_ATTR_11::Register>),
        (0x2bc => mio_pad_attr_12: ReadWrite<u32, MIO_PAD_ATTR_12::Register>),
        (0x2c0 => mio_pad_attr_13: ReadWrite<u32, MIO_PAD_ATTR_13::Register>),
        (0x2c4 => mio_pad_attr_14: ReadWrite<u32, MIO_PAD_ATTR_14::Register>),
        (0x2c8 => mio_pad_attr_15: ReadWrite<u32, MIO_PAD_ATTR_15::Register>),
        (0x2cc => mio_pad_attr_16: ReadWrite<u32, MIO_PAD_ATTR_16::Register>),
        (0x2d0 => mio_pad_attr_17: ReadWrite<u32, MIO_PAD_ATTR_17::Register>),
        (0x2d4 => mio_pad_attr_18: ReadWrite<u32, MIO_PAD_ATTR_18::Register>),
        (0x2d8 => mio_pad_attr_19: ReadWrite<u32, MIO_PAD_ATTR_19::Register>),
        (0x2dc => mio_pad_attr_20: ReadWrite<u32, MIO_PAD_ATTR_20::Register>),
        (0x2e0 => mio_pad_attr_21: ReadWrite<u32, MIO_PAD_ATTR_21::Register>),
        (0x2e4 => mio_pad_attr_22: ReadWrite<u32, MIO_PAD_ATTR_22::Register>),
        (0x2e8 => mio_pad_attr_23: ReadWrite<u32, MIO_PAD_ATTR_23::Register>),
        (0x2ec => mio_pad_attr_24: ReadWrite<u32, MIO_PAD_ATTR_24::Register>),
        (0x2f0 => mio_pad_attr_25: ReadWrite<u32, MIO_PAD_ATTR_25::Register>),
        (0x2f4 => mio_pad_attr_26: ReadWrite<u32, MIO_PAD_ATTR_26::Register>),
        (0x2f8 => mio_pad_attr_27: ReadWrite<u32, MIO_PAD_ATTR_27::Register>),
        (0x2fc => mio_pad_attr_28: ReadWrite<u32, MIO_PAD_ATTR_28::Register>),
        (0x300 => mio_pad_attr_29: ReadWrite<u32, MIO_PAD_ATTR_29::Register>),
        (0x304 => mio_pad_attr_30: ReadWrite<u32, MIO_PAD_ATTR_30::Register>),
        (0x308 => mio_pad_attr_31: ReadWrite<u32, MIO_PAD_ATTR_31::Register>),
        (0x30c => dio_pad_attr_regwen_0: ReadWrite<u32, DIO_PAD_ATTR_REGWEN_0::Register>),
        (0x310 => dio_pad_attr_regwen_1: ReadWrite<u32, DIO_PAD_ATTR_REGWEN_1::Register>),
        (0x314 => dio_pad_attr_regwen_2: ReadWrite<u32, DIO_PAD_ATTR_REGWEN_2::Register>),
        (0x318 => dio_pad_attr_regwen_3: ReadWrite<u32, DIO_PAD_ATTR_REGWEN_3::Register>),
        (0x31c => dio_pad_attr_regwen_4: ReadWrite<u32, DIO_PAD_ATTR_REGWEN_4::Register>),
        (0x320 => dio_pad_attr_regwen_5: ReadWrite<u32, DIO_PAD_ATTR_REGWEN_5::Register>),
        (0x324 => dio_pad_attr_regwen_6: ReadWrite<u32, DIO_PAD_ATTR_REGWEN_6::Register>),
        (0x328 => dio_pad_attr_regwen_7: ReadWrite<u32, DIO_PAD_ATTR_REGWEN_7::Register>),
        (0x32c => dio_pad_attr_regwen_8: ReadWrite<u32, DIO_PAD_ATTR_REGWEN_8::Register>),
        (0x330 => dio_pad_attr_regwen_9: ReadWrite<u32, DIO_PAD_ATTR_REGWEN_9::Register>),
        (0x334 => dio_pad_attr_regwen_10: ReadWrite<u32, DIO_PAD_ATTR_REGWEN_10::Register>),
        (0x338 => dio_pad_attr_regwen_11: ReadWrite<u32, DIO_PAD_ATTR_REGWEN_11::Register>),
        (0x33c => dio_pad_attr_regwen_12: ReadWrite<u32, DIO_PAD_ATTR_REGWEN_12::Register>),
        (0x340 => dio_pad_attr_regwen_13: ReadWrite<u32, DIO_PAD_ATTR_REGWEN_13::Register>),
        (0x344 => dio_pad_attr_regwen_14: ReadWrite<u32, DIO_PAD_ATTR_REGWEN_14::Register>),
        (0x348 => dio_pad_attr_regwen_15: ReadWrite<u32, DIO_PAD_ATTR_REGWEN_15::Register>),
        (0x34c => dio_pad_attr_0: ReadWrite<u32, DIO_PAD_ATTR_0::Register>),
        (0x350 => dio_pad_attr_1: ReadWrite<u32, DIO_PAD_ATTR_1::Register>),
        (0x354 => dio_pad_attr_2: ReadWrite<u32, DIO_PAD_ATTR_2::Register>),
        (0x358 => dio_pad_attr_3: ReadWrite<u32, DIO_PAD_ATTR_3::Register>),
        (0x35c => dio_pad_attr_4: ReadWrite<u32, DIO_PAD_ATTR_4::Register>),
        (0x360 => dio_pad_attr_5: ReadWrite<u32, DIO_PAD_ATTR_5::Register>),
        (0x364 => dio_pad_attr_6: ReadWrite<u32, DIO_PAD_ATTR_6::Register>),
        (0x368 => dio_pad_attr_7: ReadWrite<u32, DIO_PAD_ATTR_7::Register>),
        (0x36c => dio_pad_attr_8: ReadWrite<u32, DIO_PAD_ATTR_8::Register>),
        (0x370 => dio_pad_attr_9: ReadWrite<u32, DIO_PAD_ATTR_9::Register>),
        (0x374 => dio_pad_attr_10: ReadWrite<u32, DIO_PAD_ATTR_10::Register>),
        (0x378 => dio_pad_attr_11: ReadWrite<u32, DIO_PAD_ATTR_11::Register>),
        (0x37c => dio_pad_attr_12: ReadWrite<u32, DIO_PAD_ATTR_12::Register>),
        (0x380 => dio_pad_attr_13: ReadWrite<u32, DIO_PAD_ATTR_13::Register>),
        (0x384 => dio_pad_attr_14: ReadWrite<u32, DIO_PAD_ATTR_14::Register>),
        (0x388 => dio_pad_attr_15: ReadWrite<u32, DIO_PAD_ATTR_15::Register>),
        (0x38c => mio_pad_sleep_status: ReadWrite<u32, MIO_PAD_SLEEP_STATUS::Register>),
        (0x390 => mio_pad_sleep_regwen_0: ReadWrite<u32, MIO_PAD_SLEEP_REGWEN_0::Register>),
        (0x394 => mio_pad_sleep_regwen_1: ReadWrite<u32, MIO_PAD_SLEEP_REGWEN_1::Register>),
        (0x398 => mio_pad_sleep_regwen_2: ReadWrite<u32, MIO_PAD_SLEEP_REGWEN_2::Register>),
        (0x39c => mio_pad_sleep_regwen_3: ReadWrite<u32, MIO_PAD_SLEEP_REGWEN_3::Register>),
        (0x3a0 => mio_pad_sleep_regwen_4: ReadWrite<u32, MIO_PAD_SLEEP_REGWEN_4::Register>),
        (0x3a4 => mio_pad_sleep_regwen_5: ReadWrite<u32, MIO_PAD_SLEEP_REGWEN_5::Register>),
        (0x3a8 => mio_pad_sleep_regwen_6: ReadWrite<u32, MIO_PAD_SLEEP_REGWEN_6::Register>),
        (0x3ac => mio_pad_sleep_regwen_7: ReadWrite<u32, MIO_PAD_SLEEP_REGWEN_7::Register>),
        (0x3b0 => mio_pad_sleep_regwen_8: ReadWrite<u32, MIO_PAD_SLEEP_REGWEN_8::Register>),
        (0x3b4 => mio_pad_sleep_regwen_9: ReadWrite<u32, MIO_PAD_SLEEP_REGWEN_9::Register>),
        (0x3b8 => mio_pad_sleep_regwen_10: ReadWrite<u32, MIO_PAD_SLEEP_REGWEN_10::Register>),
        (0x3bc => mio_pad_sleep_regwen_11: ReadWrite<u32, MIO_PAD_SLEEP_REGWEN_11::Register>),
        (0x3c0 => mio_pad_sleep_regwen_12: ReadWrite<u32, MIO_PAD_SLEEP_REGWEN_12::Register>),
        (0x3c4 => mio_pad_sleep_regwen_13: ReadWrite<u32, MIO_PAD_SLEEP_REGWEN_13::Register>),
        (0x3c8 => mio_pad_sleep_regwen_14: ReadWrite<u32, MIO_PAD_SLEEP_REGWEN_14::Register>),
        (0x3cc => mio_pad_sleep_regwen_15: ReadWrite<u32, MIO_PAD_SLEEP_REGWEN_15::Register>),
        (0x3d0 => mio_pad_sleep_regwen_16: ReadWrite<u32, MIO_PAD_SLEEP_REGWEN_16::Register>),
        (0x3d4 => mio_pad_sleep_regwen_17: ReadWrite<u32, MIO_PAD_SLEEP_REGWEN_17::Register>),
        (0x3d8 => mio_pad_sleep_regwen_18: ReadWrite<u32, MIO_PAD_SLEEP_REGWEN_18::Register>),
        (0x3dc => mio_pad_sleep_regwen_19: ReadWrite<u32, MIO_PAD_SLEEP_REGWEN_19::Register>),
        (0x3e0 => mio_pad_sleep_regwen_20: ReadWrite<u32, MIO_PAD_SLEEP_REGWEN_20::Register>),
        (0x3e4 => mio_pad_sleep_regwen_21: ReadWrite<u32, MIO_PAD_SLEEP_REGWEN_21::Register>),
        (0x3e8 => mio_pad_sleep_regwen_22: ReadWrite<u32, MIO_PAD_SLEEP_REGWEN_22::Register>),
        (0x3ec => mio_pad_sleep_regwen_23: ReadWrite<u32, MIO_PAD_SLEEP_REGWEN_23::Register>),
        (0x3f0 => mio_pad_sleep_regwen_24: ReadWrite<u32, MIO_PAD_SLEEP_REGWEN_24::Register>),
        (0x3f4 => mio_pad_sleep_regwen_25: ReadWrite<u32, MIO_PAD_SLEEP_REGWEN_25::Register>),
        (0x3f8 => mio_pad_sleep_regwen_26: ReadWrite<u32, MIO_PAD_SLEEP_REGWEN_26::Register>),
        (0x3fc => mio_pad_sleep_regwen_27: ReadWrite<u32, MIO_PAD_SLEEP_REGWEN_27::Register>),
        (0x400 => mio_pad_sleep_regwen_28: ReadWrite<u32, MIO_PAD_SLEEP_REGWEN_28::Register>),
        (0x404 => mio_pad_sleep_regwen_29: ReadWrite<u32, MIO_PAD_SLEEP_REGWEN_29::Register>),
        (0x408 => mio_pad_sleep_regwen_30: ReadWrite<u32, MIO_PAD_SLEEP_REGWEN_30::Register>),
        (0x40c => mio_pad_sleep_regwen_31: ReadWrite<u32, MIO_PAD_SLEEP_REGWEN_31::Register>),
        (0x410 => mio_pad_sleep_en_0: ReadWrite<u32, MIO_PAD_SLEEP_EN_0::Register>),
        (0x414 => mio_pad_sleep_en_1: ReadWrite<u32, MIO_PAD_SLEEP_EN_1::Register>),
        (0x418 => mio_pad_sleep_en_2: ReadWrite<u32, MIO_PAD_SLEEP_EN_2::Register>),
        (0x41c => mio_pad_sleep_en_3: ReadWrite<u32, MIO_PAD_SLEEP_EN_3::Register>),
        (0x420 => mio_pad_sleep_en_4: ReadWrite<u32, MIO_PAD_SLEEP_EN_4::Register>),
        (0x424 => mio_pad_sleep_en_5: ReadWrite<u32, MIO_PAD_SLEEP_EN_5::Register>),
        (0x428 => mio_pad_sleep_en_6: ReadWrite<u32, MIO_PAD_SLEEP_EN_6::Register>),
        (0x42c => mio_pad_sleep_en_7: ReadWrite<u32, MIO_PAD_SLEEP_EN_7::Register>),
        (0x430 => mio_pad_sleep_en_8: ReadWrite<u32, MIO_PAD_SLEEP_EN_8::Register>),
        (0x434 => mio_pad_sleep_en_9: ReadWrite<u32, MIO_PAD_SLEEP_EN_9::Register>),
        (0x438 => mio_pad_sleep_en_10: ReadWrite<u32, MIO_PAD_SLEEP_EN_10::Register>),
        (0x43c => mio_pad_sleep_en_11: ReadWrite<u32, MIO_PAD_SLEEP_EN_11::Register>),
        (0x440 => mio_pad_sleep_en_12: ReadWrite<u32, MIO_PAD_SLEEP_EN_12::Register>),
        (0x444 => mio_pad_sleep_en_13: ReadWrite<u32, MIO_PAD_SLEEP_EN_13::Register>),
        (0x448 => mio_pad_sleep_en_14: ReadWrite<u32, MIO_PAD_SLEEP_EN_14::Register>),
        (0x44c => mio_pad_sleep_en_15: ReadWrite<u32, MIO_PAD_SLEEP_EN_15::Register>),
        (0x450 => mio_pad_sleep_en_16: ReadWrite<u32, MIO_PAD_SLEEP_EN_16::Register>),
        (0x454 => mio_pad_sleep_en_17: ReadWrite<u32, MIO_PAD_SLEEP_EN_17::Register>),
        (0x458 => mio_pad_sleep_en_18: ReadWrite<u32, MIO_PAD_SLEEP_EN_18::Register>),
        (0x45c => mio_pad_sleep_en_19: ReadWrite<u32, MIO_PAD_SLEEP_EN_19::Register>),
        (0x460 => mio_pad_sleep_en_20: ReadWrite<u32, MIO_PAD_SLEEP_EN_20::Register>),
        (0x464 => mio_pad_sleep_en_21: ReadWrite<u32, MIO_PAD_SLEEP_EN_21::Register>),
        (0x468 => mio_pad_sleep_en_22: ReadWrite<u32, MIO_PAD_SLEEP_EN_22::Register>),
        (0x46c => mio_pad_sleep_en_23: ReadWrite<u32, MIO_PAD_SLEEP_EN_23::Register>),
        (0x470 => mio_pad_sleep_en_24: ReadWrite<u32, MIO_PAD_SLEEP_EN_24::Register>),
        (0x474 => mio_pad_sleep_en_25: ReadWrite<u32, MIO_PAD_SLEEP_EN_25::Register>),
        (0x478 => mio_pad_sleep_en_26: ReadWrite<u32, MIO_PAD_SLEEP_EN_26::Register>),
        (0x47c => mio_pad_sleep_en_27: ReadWrite<u32, MIO_PAD_SLEEP_EN_27::Register>),
        (0x480 => mio_pad_sleep_en_28: ReadWrite<u32, MIO_PAD_SLEEP_EN_28::Register>),
        (0x484 => mio_pad_sleep_en_29: ReadWrite<u32, MIO_PAD_SLEEP_EN_29::Register>),
        (0x488 => mio_pad_sleep_en_30: ReadWrite<u32, MIO_PAD_SLEEP_EN_30::Register>),
        (0x48c => mio_pad_sleep_en_31: ReadWrite<u32, MIO_PAD_SLEEP_EN_31::Register>),
        (0x490 => mio_pad_sleep_mode_0: ReadWrite<u32, MIO_PAD_SLEEP_MODE_0::Register>),
        (0x494 => mio_pad_sleep_mode_1: ReadWrite<u32, MIO_PAD_SLEEP_MODE_1::Register>),
        (0x498 => mio_pad_sleep_mode_2: ReadWrite<u32, MIO_PAD_SLEEP_MODE_2::Register>),
        (0x49c => mio_pad_sleep_mode_3: ReadWrite<u32, MIO_PAD_SLEEP_MODE_3::Register>),
        (0x4a0 => mio_pad_sleep_mode_4: ReadWrite<u32, MIO_PAD_SLEEP_MODE_4::Register>),
        (0x4a4 => mio_pad_sleep_mode_5: ReadWrite<u32, MIO_PAD_SLEEP_MODE_5::Register>),
        (0x4a8 => mio_pad_sleep_mode_6: ReadWrite<u32, MIO_PAD_SLEEP_MODE_6::Register>),
        (0x4ac => mio_pad_sleep_mode_7: ReadWrite<u32, MIO_PAD_SLEEP_MODE_7::Register>),
        (0x4b0 => mio_pad_sleep_mode_8: ReadWrite<u32, MIO_PAD_SLEEP_MODE_8::Register>),
        (0x4b4 => mio_pad_sleep_mode_9: ReadWrite<u32, MIO_PAD_SLEEP_MODE_9::Register>),
        (0x4b8 => mio_pad_sleep_mode_10: ReadWrite<u32, MIO_PAD_SLEEP_MODE_10::Register>),
        (0x4bc => mio_pad_sleep_mode_11: ReadWrite<u32, MIO_PAD_SLEEP_MODE_11::Register>),
        (0x4c0 => mio_pad_sleep_mode_12: ReadWrite<u32, MIO_PAD_SLEEP_MODE_12::Register>),
        (0x4c4 => mio_pad_sleep_mode_13: ReadWrite<u32, MIO_PAD_SLEEP_MODE_13::Register>),
        (0x4c8 => mio_pad_sleep_mode_14: ReadWrite<u32, MIO_PAD_SLEEP_MODE_14::Register>),
        (0x4cc => mio_pad_sleep_mode_15: ReadWrite<u32, MIO_PAD_SLEEP_MODE_15::Register>),
        (0x4d0 => mio_pad_sleep_mode_16: ReadWrite<u32, MIO_PAD_SLEEP_MODE_16::Register>),
        (0x4d4 => mio_pad_sleep_mode_17: ReadWrite<u32, MIO_PAD_SLEEP_MODE_17::Register>),
        (0x4d8 => mio_pad_sleep_mode_18: ReadWrite<u32, MIO_PAD_SLEEP_MODE_18::Register>),
        (0x4dc => mio_pad_sleep_mode_19: ReadWrite<u32, MIO_PAD_SLEEP_MODE_19::Register>),
        (0x4e0 => mio_pad_sleep_mode_20: ReadWrite<u32, MIO_PAD_SLEEP_MODE_20::Register>),
        (0x4e4 => mio_pad_sleep_mode_21: ReadWrite<u32, MIO_PAD_SLEEP_MODE_21::Register>),
        (0x4e8 => mio_pad_sleep_mode_22: ReadWrite<u32, MIO_PAD_SLEEP_MODE_22::Register>),
        (0x4ec => mio_pad_sleep_mode_23: ReadWrite<u32, MIO_PAD_SLEEP_MODE_23::Register>),
        (0x4f0 => mio_pad_sleep_mode_24: ReadWrite<u32, MIO_PAD_SLEEP_MODE_24::Register>),
        (0x4f4 => mio_pad_sleep_mode_25: ReadWrite<u32, MIO_PAD_SLEEP_MODE_25::Register>),
        (0x4f8 => mio_pad_sleep_mode_26: ReadWrite<u32, MIO_PAD_SLEEP_MODE_26::Register>),
        (0x4fc => mio_pad_sleep_mode_27: ReadWrite<u32, MIO_PAD_SLEEP_MODE_27::Register>),
        (0x500 => mio_pad_sleep_mode_28: ReadWrite<u32, MIO_PAD_SLEEP_MODE_28::Register>),
        (0x504 => mio_pad_sleep_mode_29: ReadWrite<u32, MIO_PAD_SLEEP_MODE_29::Register>),
        (0x508 => mio_pad_sleep_mode_30: ReadWrite<u32, MIO_PAD_SLEEP_MODE_30::Register>),
        (0x50c => mio_pad_sleep_mode_31: ReadWrite<u32, MIO_PAD_SLEEP_MODE_31::Register>),
        (0x510 => dio_pad_sleep_status: ReadWrite<u32, DIO_PAD_SLEEP_STATUS::Register>),
        (0x514 => dio_pad_sleep_regwen_0: ReadWrite<u32, DIO_PAD_SLEEP_REGWEN_0::Register>),
        (0x518 => dio_pad_sleep_regwen_1: ReadWrite<u32, DIO_PAD_SLEEP_REGWEN_1::Register>),
        (0x51c => dio_pad_sleep_regwen_2: ReadWrite<u32, DIO_PAD_SLEEP_REGWEN_2::Register>),
        (0x520 => dio_pad_sleep_regwen_3: ReadWrite<u32, DIO_PAD_SLEEP_REGWEN_3::Register>),
        (0x524 => dio_pad_sleep_regwen_4: ReadWrite<u32, DIO_PAD_SLEEP_REGWEN_4::Register>),
        (0x528 => dio_pad_sleep_regwen_5: ReadWrite<u32, DIO_PAD_SLEEP_REGWEN_5::Register>),
        (0x52c => dio_pad_sleep_regwen_6: ReadWrite<u32, DIO_PAD_SLEEP_REGWEN_6::Register>),
        (0x530 => dio_pad_sleep_regwen_7: ReadWrite<u32, DIO_PAD_SLEEP_REGWEN_7::Register>),
        (0x534 => dio_pad_sleep_regwen_8: ReadWrite<u32, DIO_PAD_SLEEP_REGWEN_8::Register>),
        (0x538 => dio_pad_sleep_regwen_9: ReadWrite<u32, DIO_PAD_SLEEP_REGWEN_9::Register>),
        (0x53c => dio_pad_sleep_regwen_10: ReadWrite<u32, DIO_PAD_SLEEP_REGWEN_10::Register>),
        (0x540 => dio_pad_sleep_regwen_11: ReadWrite<u32, DIO_PAD_SLEEP_REGWEN_11::Register>),
        (0x544 => dio_pad_sleep_regwen_12: ReadWrite<u32, DIO_PAD_SLEEP_REGWEN_12::Register>),
        (0x548 => dio_pad_sleep_regwen_13: ReadWrite<u32, DIO_PAD_SLEEP_REGWEN_13::Register>),
        (0x54c => dio_pad_sleep_regwen_14: ReadWrite<u32, DIO_PAD_SLEEP_REGWEN_14::Register>),
        (0x550 => dio_pad_sleep_regwen_15: ReadWrite<u32, DIO_PAD_SLEEP_REGWEN_15::Register>),
        (0x554 => dio_pad_sleep_en_0: ReadWrite<u32, DIO_PAD_SLEEP_EN_0::Register>),
        (0x558 => dio_pad_sleep_en_1: ReadWrite<u32, DIO_PAD_SLEEP_EN_1::Register>),
        (0x55c => dio_pad_sleep_en_2: ReadWrite<u32, DIO_PAD_SLEEP_EN_2::Register>),
        (0x560 => dio_pad_sleep_en_3: ReadWrite<u32, DIO_PAD_SLEEP_EN_3::Register>),
        (0x564 => dio_pad_sleep_en_4: ReadWrite<u32, DIO_PAD_SLEEP_EN_4::Register>),
        (0x568 => dio_pad_sleep_en_5: ReadWrite<u32, DIO_PAD_SLEEP_EN_5::Register>),
        (0x56c => dio_pad_sleep_en_6: ReadWrite<u32, DIO_PAD_SLEEP_EN_6::Register>),
        (0x570 => dio_pad_sleep_en_7: ReadWrite<u32, DIO_PAD_SLEEP_EN_7::Register>),
        (0x574 => dio_pad_sleep_en_8: ReadWrite<u32, DIO_PAD_SLEEP_EN_8::Register>),
        (0x578 => dio_pad_sleep_en_9: ReadWrite<u32, DIO_PAD_SLEEP_EN_9::Register>),
        (0x57c => dio_pad_sleep_en_10: ReadWrite<u32, DIO_PAD_SLEEP_EN_10::Register>),
        (0x580 => dio_pad_sleep_en_11: ReadWrite<u32, DIO_PAD_SLEEP_EN_11::Register>),
        (0x584 => dio_pad_sleep_en_12: ReadWrite<u32, DIO_PAD_SLEEP_EN_12::Register>),
        (0x588 => dio_pad_sleep_en_13: ReadWrite<u32, DIO_PAD_SLEEP_EN_13::Register>),
        (0x58c => dio_pad_sleep_en_14: ReadWrite<u32, DIO_PAD_SLEEP_EN_14::Register>),
        (0x590 => dio_pad_sleep_en_15: ReadWrite<u32, DIO_PAD_SLEEP_EN_15::Register>),
        (0x594 => dio_pad_sleep_mode_0: ReadWrite<u32, DIO_PAD_SLEEP_MODE_0::Register>),
        (0x598 => dio_pad_sleep_mode_1: ReadWrite<u32, DIO_PAD_SLEEP_MODE_1::Register>),
        (0x59c => dio_pad_sleep_mode_2: ReadWrite<u32, DIO_PAD_SLEEP_MODE_2::Register>),
        (0x5a0 => dio_pad_sleep_mode_3: ReadWrite<u32, DIO_PAD_SLEEP_MODE_3::Register>),
        (0x5a4 => dio_pad_sleep_mode_4: ReadWrite<u32, DIO_PAD_SLEEP_MODE_4::Register>),
        (0x5a8 => dio_pad_sleep_mode_5: ReadWrite<u32, DIO_PAD_SLEEP_MODE_5::Register>),
        (0x5ac => dio_pad_sleep_mode_6: ReadWrite<u32, DIO_PAD_SLEEP_MODE_6::Register>),
        (0x5b0 => dio_pad_sleep_mode_7: ReadWrite<u32, DIO_PAD_SLEEP_MODE_7::Register>),
        (0x5b4 => dio_pad_sleep_mode_8: ReadWrite<u32, DIO_PAD_SLEEP_MODE_8::Register>),
        (0x5b8 => dio_pad_sleep_mode_9: ReadWrite<u32, DIO_PAD_SLEEP_MODE_9::Register>),
        (0x5bc => dio_pad_sleep_mode_10: ReadWrite<u32, DIO_PAD_SLEEP_MODE_10::Register>),
        (0x5c0 => dio_pad_sleep_mode_11: ReadWrite<u32, DIO_PAD_SLEEP_MODE_11::Register>),
        (0x5c4 => dio_pad_sleep_mode_12: ReadWrite<u32, DIO_PAD_SLEEP_MODE_12::Register>),
        (0x5c8 => dio_pad_sleep_mode_13: ReadWrite<u32, DIO_PAD_SLEEP_MODE_13::Register>),
        (0x5cc => dio_pad_sleep_mode_14: ReadWrite<u32, DIO_PAD_SLEEP_MODE_14::Register>),
        (0x5d0 => dio_pad_sleep_mode_15: ReadWrite<u32, DIO_PAD_SLEEP_MODE_15::Register>),
        (0x5d4 => wkup_detector_regwen_0: ReadWrite<u32, WKUP_DETECTOR_REGWEN_0::Register>),
        (0x5d8 => wkup_detector_regwen_1: ReadWrite<u32, WKUP_DETECTOR_REGWEN_1::Register>),
        (0x5dc => wkup_detector_regwen_2: ReadWrite<u32, WKUP_DETECTOR_REGWEN_2::Register>),
        (0x5e0 => wkup_detector_regwen_3: ReadWrite<u32, WKUP_DETECTOR_REGWEN_3::Register>),
        (0x5e4 => wkup_detector_regwen_4: ReadWrite<u32, WKUP_DETECTOR_REGWEN_4::Register>),
        (0x5e8 => wkup_detector_regwen_5: ReadWrite<u32, WKUP_DETECTOR_REGWEN_5::Register>),
        (0x5ec => wkup_detector_regwen_6: ReadWrite<u32, WKUP_DETECTOR_REGWEN_6::Register>),
        (0x5f0 => wkup_detector_regwen_7: ReadWrite<u32, WKUP_DETECTOR_REGWEN_7::Register>),
        (0x5f4 => wkup_detector_en_0: ReadWrite<u32, WKUP_DETECTOR_EN_0::Register>),
        (0x5f8 => wkup_detector_en_1: ReadWrite<u32, WKUP_DETECTOR_EN_1::Register>),
        (0x5fc => wkup_detector_en_2: ReadWrite<u32, WKUP_DETECTOR_EN_2::Register>),
        (0x600 => wkup_detector_en_3: ReadWrite<u32, WKUP_DETECTOR_EN_3::Register>),
        (0x604 => wkup_detector_en_4: ReadWrite<u32, WKUP_DETECTOR_EN_4::Register>),
        (0x608 => wkup_detector_en_5: ReadWrite<u32, WKUP_DETECTOR_EN_5::Register>),
        (0x60c => wkup_detector_en_6: ReadWrite<u32, WKUP_DETECTOR_EN_6::Register>),
        (0x610 => wkup_detector_en_7: ReadWrite<u32, WKUP_DETECTOR_EN_7::Register>),
        (0x614 => wkup_detector_0: ReadWrite<u32, WKUP_DETECTOR_0::Register>),
        (0x618 => wkup_detector_1: ReadWrite<u32, WKUP_DETECTOR_1::Register>),
        (0x61c => wkup_detector_2: ReadWrite<u32, WKUP_DETECTOR_2::Register>),
        (0x620 => wkup_detector_3: ReadWrite<u32, WKUP_DETECTOR_3::Register>),
        (0x624 => wkup_detector_4: ReadWrite<u32, WKUP_DETECTOR_4::Register>),
        (0x628 => wkup_detector_5: ReadWrite<u32, WKUP_DETECTOR_5::Register>),
        (0x62c => wkup_detector_6: ReadWrite<u32, WKUP_DETECTOR_6::Register>),
        (0x630 => wkup_detector_7: ReadWrite<u32, WKUP_DETECTOR_7::Register>),
        (0x634 => wkup_detector_cnt_th_0: ReadWrite<u32, WKUP_DETECTOR_CNT_TH_0::Register>),
        (0x638 => wkup_detector_cnt_th_1: ReadWrite<u32, WKUP_DETECTOR_CNT_TH_1::Register>),
        (0x63c => wkup_detector_cnt_th_2: ReadWrite<u32, WKUP_DETECTOR_CNT_TH_2::Register>),
        (0x640 => wkup_detector_cnt_th_3: ReadWrite<u32, WKUP_DETECTOR_CNT_TH_3::Register>),
        (0x644 => wkup_detector_cnt_th_4: ReadWrite<u32, WKUP_DETECTOR_CNT_TH_4::Register>),
        (0x648 => wkup_detector_cnt_th_5: ReadWrite<u32, WKUP_DETECTOR_CNT_TH_5::Register>),
        (0x64c => wkup_detector_cnt_th_6: ReadWrite<u32, WKUP_DETECTOR_CNT_TH_6::Register>),
        (0x650 => wkup_detector_cnt_th_7: ReadWrite<u32, WKUP_DETECTOR_CNT_TH_7::Register>),
        (0x654 => wkup_detector_padsel_0: ReadWrite<u32, WKUP_DETECTOR_PADSEL_0::Register>),
        (0x658 => wkup_detector_padsel_1: ReadWrite<u32, WKUP_DETECTOR_PADSEL_1::Register>),
        (0x65c => wkup_detector_padsel_2: ReadWrite<u32, WKUP_DETECTOR_PADSEL_2::Register>),
        (0x660 => wkup_detector_padsel_3: ReadWrite<u32, WKUP_DETECTOR_PADSEL_3::Register>),
        (0x664 => wkup_detector_padsel_4: ReadWrite<u32, WKUP_DETECTOR_PADSEL_4::Register>),
        (0x668 => wkup_detector_padsel_5: ReadWrite<u32, WKUP_DETECTOR_PADSEL_5::Register>),
        (0x66c => wkup_detector_padsel_6: ReadWrite<u32, WKUP_DETECTOR_PADSEL_6::Register>),
        (0x670 => wkup_detector_padsel_7: ReadWrite<u32, WKUP_DETECTOR_PADSEL_7::Register>),
        (0x674 => wkup_cause: ReadWrite<u32, WKUP_CAUSE::Register>),
    }
}

register_bitfields![u32,
    ALERT_TEST [
        FATAL_FAULT OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PERIPH_INSEL_REGWEN_0 [
        EN_0 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PERIPH_INSEL_REGWEN_1 [
        EN_1 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PERIPH_INSEL_REGWEN_2 [
        EN_2 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PERIPH_INSEL_REGWEN_3 [
        EN_3 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PERIPH_INSEL_REGWEN_4 [
        EN_4 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PERIPH_INSEL_REGWEN_5 [
        EN_5 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PERIPH_INSEL_REGWEN_6 [
        EN_6 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PERIPH_INSEL_REGWEN_7 [
        EN_7 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PERIPH_INSEL_REGWEN_8 [
        EN_8 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PERIPH_INSEL_REGWEN_9 [
        EN_9 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PERIPH_INSEL_REGWEN_10 [
        EN_10 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PERIPH_INSEL_REGWEN_11 [
        EN_11 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PERIPH_INSEL_REGWEN_12 [
        EN_12 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PERIPH_INSEL_REGWEN_13 [
        EN_13 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PERIPH_INSEL_REGWEN_14 [
        EN_14 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PERIPH_INSEL_REGWEN_15 [
        EN_15 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PERIPH_INSEL_REGWEN_16 [
        EN_16 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PERIPH_INSEL_REGWEN_17 [
        EN_17 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PERIPH_INSEL_REGWEN_18 [
        EN_18 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PERIPH_INSEL_REGWEN_19 [
        EN_19 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PERIPH_INSEL_REGWEN_20 [
        EN_20 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PERIPH_INSEL_REGWEN_21 [
        EN_21 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PERIPH_INSEL_REGWEN_22 [
        EN_22 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PERIPH_INSEL_REGWEN_23 [
        EN_23 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PERIPH_INSEL_REGWEN_24 [
        EN_24 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PERIPH_INSEL_REGWEN_25 [
        EN_25 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PERIPH_INSEL_REGWEN_26 [
        EN_26 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PERIPH_INSEL_REGWEN_27 [
        EN_27 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PERIPH_INSEL_REGWEN_28 [
        EN_28 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PERIPH_INSEL_REGWEN_29 [
        EN_29 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PERIPH_INSEL_REGWEN_30 [
        EN_30 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PERIPH_INSEL_REGWEN_31 [
        EN_31 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PERIPH_INSEL_REGWEN_32 [
        EN_32 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PERIPH_INSEL_0 [
        IN_0 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_PERIPH_INSEL_1 [
        IN_1 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_PERIPH_INSEL_2 [
        IN_2 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_PERIPH_INSEL_3 [
        IN_3 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_PERIPH_INSEL_4 [
        IN_4 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_PERIPH_INSEL_5 [
        IN_5 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_PERIPH_INSEL_6 [
        IN_6 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_PERIPH_INSEL_7 [
        IN_7 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_PERIPH_INSEL_8 [
        IN_8 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_PERIPH_INSEL_9 [
        IN_9 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_PERIPH_INSEL_10 [
        IN_10 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_PERIPH_INSEL_11 [
        IN_11 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_PERIPH_INSEL_12 [
        IN_12 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_PERIPH_INSEL_13 [
        IN_13 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_PERIPH_INSEL_14 [
        IN_14 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_PERIPH_INSEL_15 [
        IN_15 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_PERIPH_INSEL_16 [
        IN_16 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_PERIPH_INSEL_17 [
        IN_17 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_PERIPH_INSEL_18 [
        IN_18 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_PERIPH_INSEL_19 [
        IN_19 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_PERIPH_INSEL_20 [
        IN_20 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_PERIPH_INSEL_21 [
        IN_21 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_PERIPH_INSEL_22 [
        IN_22 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_PERIPH_INSEL_23 [
        IN_23 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_PERIPH_INSEL_24 [
        IN_24 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_PERIPH_INSEL_25 [
        IN_25 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_PERIPH_INSEL_26 [
        IN_26 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_PERIPH_INSEL_27 [
        IN_27 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_PERIPH_INSEL_28 [
        IN_28 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_PERIPH_INSEL_29 [
        IN_29 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_PERIPH_INSEL_30 [
        IN_30 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_PERIPH_INSEL_31 [
        IN_31 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_PERIPH_INSEL_32 [
        IN_32 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_OUTSEL_REGWEN_0 [
        EN_0 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_OUTSEL_REGWEN_1 [
        EN_1 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_OUTSEL_REGWEN_2 [
        EN_2 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_OUTSEL_REGWEN_3 [
        EN_3 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_OUTSEL_REGWEN_4 [
        EN_4 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_OUTSEL_REGWEN_5 [
        EN_5 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_OUTSEL_REGWEN_6 [
        EN_6 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_OUTSEL_REGWEN_7 [
        EN_7 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_OUTSEL_REGWEN_8 [
        EN_8 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_OUTSEL_REGWEN_9 [
        EN_9 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_OUTSEL_REGWEN_10 [
        EN_10 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_OUTSEL_REGWEN_11 [
        EN_11 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_OUTSEL_REGWEN_12 [
        EN_12 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_OUTSEL_REGWEN_13 [
        EN_13 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_OUTSEL_REGWEN_14 [
        EN_14 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_OUTSEL_REGWEN_15 [
        EN_15 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_OUTSEL_REGWEN_16 [
        EN_16 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_OUTSEL_REGWEN_17 [
        EN_17 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_OUTSEL_REGWEN_18 [
        EN_18 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_OUTSEL_REGWEN_19 [
        EN_19 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_OUTSEL_REGWEN_20 [
        EN_20 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_OUTSEL_REGWEN_21 [
        EN_21 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_OUTSEL_REGWEN_22 [
        EN_22 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_OUTSEL_REGWEN_23 [
        EN_23 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_OUTSEL_REGWEN_24 [
        EN_24 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_OUTSEL_REGWEN_25 [
        EN_25 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_OUTSEL_REGWEN_26 [
        EN_26 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_OUTSEL_REGWEN_27 [
        EN_27 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_OUTSEL_REGWEN_28 [
        EN_28 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_OUTSEL_REGWEN_29 [
        EN_29 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_OUTSEL_REGWEN_30 [
        EN_30 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_OUTSEL_REGWEN_31 [
        EN_31 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_OUTSEL_0 [
        OUT_0 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_OUTSEL_1 [
        OUT_1 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_OUTSEL_2 [
        OUT_2 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_OUTSEL_3 [
        OUT_3 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_OUTSEL_4 [
        OUT_4 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_OUTSEL_5 [
        OUT_5 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_OUTSEL_6 [
        OUT_6 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_OUTSEL_7 [
        OUT_7 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_OUTSEL_8 [
        OUT_8 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_OUTSEL_9 [
        OUT_9 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_OUTSEL_10 [
        OUT_10 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_OUTSEL_11 [
        OUT_11 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_OUTSEL_12 [
        OUT_12 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_OUTSEL_13 [
        OUT_13 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_OUTSEL_14 [
        OUT_14 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_OUTSEL_15 [
        OUT_15 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_OUTSEL_16 [
        OUT_16 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_OUTSEL_17 [
        OUT_17 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_OUTSEL_18 [
        OUT_18 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_OUTSEL_19 [
        OUT_19 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_OUTSEL_20 [
        OUT_20 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_OUTSEL_21 [
        OUT_21 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_OUTSEL_22 [
        OUT_22 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_OUTSEL_23 [
        OUT_23 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_OUTSEL_24 [
        OUT_24 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_OUTSEL_25 [
        OUT_25 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_OUTSEL_26 [
        OUT_26 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_OUTSEL_27 [
        OUT_27 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_OUTSEL_28 [
        OUT_28 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_OUTSEL_29 [
        OUT_29 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_OUTSEL_30 [
        OUT_30 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_OUTSEL_31 [
        OUT_31 OFFSET(0) NUMBITS(6) [],
    ],
    MIO_PAD_ATTR_REGWEN_0 [
        EN_0 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_ATTR_REGWEN_1 [
        EN_1 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_ATTR_REGWEN_2 [
        EN_2 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_ATTR_REGWEN_3 [
        EN_3 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_ATTR_REGWEN_4 [
        EN_4 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_ATTR_REGWEN_5 [
        EN_5 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_ATTR_REGWEN_6 [
        EN_6 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_ATTR_REGWEN_7 [
        EN_7 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_ATTR_REGWEN_8 [
        EN_8 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_ATTR_REGWEN_9 [
        EN_9 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_ATTR_REGWEN_10 [
        EN_10 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_ATTR_REGWEN_11 [
        EN_11 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_ATTR_REGWEN_12 [
        EN_12 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_ATTR_REGWEN_13 [
        EN_13 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_ATTR_REGWEN_14 [
        EN_14 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_ATTR_REGWEN_15 [
        EN_15 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_ATTR_REGWEN_16 [
        EN_16 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_ATTR_REGWEN_17 [
        EN_17 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_ATTR_REGWEN_18 [
        EN_18 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_ATTR_REGWEN_19 [
        EN_19 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_ATTR_REGWEN_20 [
        EN_20 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_ATTR_REGWEN_21 [
        EN_21 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_ATTR_REGWEN_22 [
        EN_22 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_ATTR_REGWEN_23 [
        EN_23 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_ATTR_REGWEN_24 [
        EN_24 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_ATTR_REGWEN_25 [
        EN_25 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_ATTR_REGWEN_26 [
        EN_26 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_ATTR_REGWEN_27 [
        EN_27 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_ATTR_REGWEN_28 [
        EN_28 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_ATTR_REGWEN_29 [
        EN_29 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_ATTR_REGWEN_30 [
        EN_30 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_ATTR_REGWEN_31 [
        EN_31 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_ATTR_0 [
        ATTR_0 OFFSET(0) NUMBITS(13) [],
    ],
    MIO_PAD_ATTR_1 [
        ATTR_1 OFFSET(0) NUMBITS(13) [],
    ],
    MIO_PAD_ATTR_2 [
        ATTR_2 OFFSET(0) NUMBITS(13) [],
    ],
    MIO_PAD_ATTR_3 [
        ATTR_3 OFFSET(0) NUMBITS(13) [],
    ],
    MIO_PAD_ATTR_4 [
        ATTR_4 OFFSET(0) NUMBITS(13) [],
    ],
    MIO_PAD_ATTR_5 [
        ATTR_5 OFFSET(0) NUMBITS(13) [],
    ],
    MIO_PAD_ATTR_6 [
        ATTR_6 OFFSET(0) NUMBITS(13) [],
    ],
    MIO_PAD_ATTR_7 [
        ATTR_7 OFFSET(0) NUMBITS(13) [],
    ],
    MIO_PAD_ATTR_8 [
        ATTR_8 OFFSET(0) NUMBITS(13) [],
    ],
    MIO_PAD_ATTR_9 [
        ATTR_9 OFFSET(0) NUMBITS(13) [],
    ],
    MIO_PAD_ATTR_10 [
        ATTR_10 OFFSET(0) NUMBITS(13) [],
    ],
    MIO_PAD_ATTR_11 [
        ATTR_11 OFFSET(0) NUMBITS(13) [],
    ],
    MIO_PAD_ATTR_12 [
        ATTR_12 OFFSET(0) NUMBITS(13) [],
    ],
    MIO_PAD_ATTR_13 [
        ATTR_13 OFFSET(0) NUMBITS(13) [],
    ],
    MIO_PAD_ATTR_14 [
        ATTR_14 OFFSET(0) NUMBITS(13) [],
    ],
    MIO_PAD_ATTR_15 [
        ATTR_15 OFFSET(0) NUMBITS(13) [],
    ],
    MIO_PAD_ATTR_16 [
        ATTR_16 OFFSET(0) NUMBITS(13) [],
    ],
    MIO_PAD_ATTR_17 [
        ATTR_17 OFFSET(0) NUMBITS(13) [],
    ],
    MIO_PAD_ATTR_18 [
        ATTR_18 OFFSET(0) NUMBITS(13) [],
    ],
    MIO_PAD_ATTR_19 [
        ATTR_19 OFFSET(0) NUMBITS(13) [],
    ],
    MIO_PAD_ATTR_20 [
        ATTR_20 OFFSET(0) NUMBITS(13) [],
    ],
    MIO_PAD_ATTR_21 [
        ATTR_21 OFFSET(0) NUMBITS(13) [],
    ],
    MIO_PAD_ATTR_22 [
        ATTR_22 OFFSET(0) NUMBITS(13) [],
    ],
    MIO_PAD_ATTR_23 [
        ATTR_23 OFFSET(0) NUMBITS(13) [],
    ],
    MIO_PAD_ATTR_24 [
        ATTR_24 OFFSET(0) NUMBITS(13) [],
    ],
    MIO_PAD_ATTR_25 [
        ATTR_25 OFFSET(0) NUMBITS(13) [],
    ],
    MIO_PAD_ATTR_26 [
        ATTR_26 OFFSET(0) NUMBITS(13) [],
    ],
    MIO_PAD_ATTR_27 [
        ATTR_27 OFFSET(0) NUMBITS(13) [],
    ],
    MIO_PAD_ATTR_28 [
        ATTR_28 OFFSET(0) NUMBITS(13) [],
    ],
    MIO_PAD_ATTR_29 [
        ATTR_29 OFFSET(0) NUMBITS(13) [],
    ],
    MIO_PAD_ATTR_30 [
        ATTR_30 OFFSET(0) NUMBITS(13) [],
    ],
    MIO_PAD_ATTR_31 [
        ATTR_31 OFFSET(0) NUMBITS(13) [],
    ],
    DIO_PAD_ATTR_REGWEN_0 [
        EN_0 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_ATTR_REGWEN_1 [
        EN_1 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_ATTR_REGWEN_2 [
        EN_2 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_ATTR_REGWEN_3 [
        EN_3 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_ATTR_REGWEN_4 [
        EN_4 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_ATTR_REGWEN_5 [
        EN_5 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_ATTR_REGWEN_6 [
        EN_6 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_ATTR_REGWEN_7 [
        EN_7 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_ATTR_REGWEN_8 [
        EN_8 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_ATTR_REGWEN_9 [
        EN_9 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_ATTR_REGWEN_10 [
        EN_10 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_ATTR_REGWEN_11 [
        EN_11 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_ATTR_REGWEN_12 [
        EN_12 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_ATTR_REGWEN_13 [
        EN_13 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_ATTR_REGWEN_14 [
        EN_14 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_ATTR_REGWEN_15 [
        EN_15 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_ATTR_0 [
        ATTR_0 OFFSET(0) NUMBITS(13) [],
    ],
    DIO_PAD_ATTR_1 [
        ATTR_1 OFFSET(0) NUMBITS(13) [],
    ],
    DIO_PAD_ATTR_2 [
        ATTR_2 OFFSET(0) NUMBITS(13) [],
    ],
    DIO_PAD_ATTR_3 [
        ATTR_3 OFFSET(0) NUMBITS(13) [],
    ],
    DIO_PAD_ATTR_4 [
        ATTR_4 OFFSET(0) NUMBITS(13) [],
    ],
    DIO_PAD_ATTR_5 [
        ATTR_5 OFFSET(0) NUMBITS(13) [],
    ],
    DIO_PAD_ATTR_6 [
        ATTR_6 OFFSET(0) NUMBITS(13) [],
    ],
    DIO_PAD_ATTR_7 [
        ATTR_7 OFFSET(0) NUMBITS(13) [],
    ],
    DIO_PAD_ATTR_8 [
        ATTR_8 OFFSET(0) NUMBITS(13) [],
    ],
    DIO_PAD_ATTR_9 [
        ATTR_9 OFFSET(0) NUMBITS(13) [],
    ],
    DIO_PAD_ATTR_10 [
        ATTR_10 OFFSET(0) NUMBITS(13) [],
    ],
    DIO_PAD_ATTR_11 [
        ATTR_11 OFFSET(0) NUMBITS(13) [],
    ],
    DIO_PAD_ATTR_12 [
        ATTR_12 OFFSET(0) NUMBITS(13) [],
    ],
    DIO_PAD_ATTR_13 [
        ATTR_13 OFFSET(0) NUMBITS(13) [],
    ],
    DIO_PAD_ATTR_14 [
        ATTR_14 OFFSET(0) NUMBITS(13) [],
    ],
    DIO_PAD_ATTR_15 [
        ATTR_15 OFFSET(0) NUMBITS(13) [],
    ],
    MIO_PAD_SLEEP_STATUS [
        EN_0 OFFSET(0) NUMBITS(1) [],
        EN_1 OFFSET(1) NUMBITS(1) [],
        EN_2 OFFSET(2) NUMBITS(1) [],
        EN_3 OFFSET(3) NUMBITS(1) [],
        EN_4 OFFSET(4) NUMBITS(1) [],
        EN_5 OFFSET(5) NUMBITS(1) [],
        EN_6 OFFSET(6) NUMBITS(1) [],
        EN_7 OFFSET(7) NUMBITS(1) [],
        EN_8 OFFSET(8) NUMBITS(1) [],
        EN_9 OFFSET(9) NUMBITS(1) [],
        EN_10 OFFSET(10) NUMBITS(1) [],
        EN_11 OFFSET(11) NUMBITS(1) [],
        EN_12 OFFSET(12) NUMBITS(1) [],
        EN_13 OFFSET(13) NUMBITS(1) [],
        EN_14 OFFSET(14) NUMBITS(1) [],
        EN_15 OFFSET(15) NUMBITS(1) [],
        EN_16 OFFSET(16) NUMBITS(1) [],
        EN_17 OFFSET(17) NUMBITS(1) [],
        EN_18 OFFSET(18) NUMBITS(1) [],
        EN_19 OFFSET(19) NUMBITS(1) [],
        EN_20 OFFSET(20) NUMBITS(1) [],
        EN_21 OFFSET(21) NUMBITS(1) [],
        EN_22 OFFSET(22) NUMBITS(1) [],
        EN_23 OFFSET(23) NUMBITS(1) [],
        EN_24 OFFSET(24) NUMBITS(1) [],
        EN_25 OFFSET(25) NUMBITS(1) [],
        EN_26 OFFSET(26) NUMBITS(1) [],
        EN_27 OFFSET(27) NUMBITS(1) [],
        EN_28 OFFSET(28) NUMBITS(1) [],
        EN_29 OFFSET(29) NUMBITS(1) [],
        EN_30 OFFSET(30) NUMBITS(1) [],
        EN_31 OFFSET(31) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_REGWEN_0 [
        EN_0 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_REGWEN_1 [
        EN_1 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_REGWEN_2 [
        EN_2 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_REGWEN_3 [
        EN_3 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_REGWEN_4 [
        EN_4 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_REGWEN_5 [
        EN_5 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_REGWEN_6 [
        EN_6 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_REGWEN_7 [
        EN_7 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_REGWEN_8 [
        EN_8 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_REGWEN_9 [
        EN_9 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_REGWEN_10 [
        EN_10 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_REGWEN_11 [
        EN_11 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_REGWEN_12 [
        EN_12 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_REGWEN_13 [
        EN_13 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_REGWEN_14 [
        EN_14 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_REGWEN_15 [
        EN_15 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_REGWEN_16 [
        EN_16 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_REGWEN_17 [
        EN_17 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_REGWEN_18 [
        EN_18 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_REGWEN_19 [
        EN_19 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_REGWEN_20 [
        EN_20 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_REGWEN_21 [
        EN_21 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_REGWEN_22 [
        EN_22 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_REGWEN_23 [
        EN_23 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_REGWEN_24 [
        EN_24 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_REGWEN_25 [
        EN_25 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_REGWEN_26 [
        EN_26 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_REGWEN_27 [
        EN_27 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_REGWEN_28 [
        EN_28 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_REGWEN_29 [
        EN_29 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_REGWEN_30 [
        EN_30 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_REGWEN_31 [
        EN_31 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_EN_0 [
        EN_0 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_EN_1 [
        EN_1 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_EN_2 [
        EN_2 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_EN_3 [
        EN_3 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_EN_4 [
        EN_4 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_EN_5 [
        EN_5 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_EN_6 [
        EN_6 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_EN_7 [
        EN_7 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_EN_8 [
        EN_8 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_EN_9 [
        EN_9 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_EN_10 [
        EN_10 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_EN_11 [
        EN_11 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_EN_12 [
        EN_12 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_EN_13 [
        EN_13 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_EN_14 [
        EN_14 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_EN_15 [
        EN_15 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_EN_16 [
        EN_16 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_EN_17 [
        EN_17 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_EN_18 [
        EN_18 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_EN_19 [
        EN_19 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_EN_20 [
        EN_20 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_EN_21 [
        EN_21 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_EN_22 [
        EN_22 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_EN_23 [
        EN_23 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_EN_24 [
        EN_24 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_EN_25 [
        EN_25 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_EN_26 [
        EN_26 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_EN_27 [
        EN_27 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_EN_28 [
        EN_28 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_EN_29 [
        EN_29 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_EN_30 [
        EN_30 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_EN_31 [
        EN_31 OFFSET(0) NUMBITS(1) [],
    ],
    MIO_PAD_SLEEP_MODE_0 [
        OUT_0 OFFSET(0) NUMBITS(2) [
            TIE_LOW = 0,
            TIE_HIGH = 1,
            HIGH_Z = 2,
            KEEP = 3,
        ],
    ],
    MIO_PAD_SLEEP_MODE_1 [
        OUT_1 OFFSET(0) NUMBITS(2) [],
    ],
    MIO_PAD_SLEEP_MODE_2 [
        OUT_2 OFFSET(0) NUMBITS(2) [],
    ],
    MIO_PAD_SLEEP_MODE_3 [
        OUT_3 OFFSET(0) NUMBITS(2) [],
    ],
    MIO_PAD_SLEEP_MODE_4 [
        OUT_4 OFFSET(0) NUMBITS(2) [],
    ],
    MIO_PAD_SLEEP_MODE_5 [
        OUT_5 OFFSET(0) NUMBITS(2) [],
    ],
    MIO_PAD_SLEEP_MODE_6 [
        OUT_6 OFFSET(0) NUMBITS(2) [],
    ],
    MIO_PAD_SLEEP_MODE_7 [
        OUT_7 OFFSET(0) NUMBITS(2) [],
    ],
    MIO_PAD_SLEEP_MODE_8 [
        OUT_8 OFFSET(0) NUMBITS(2) [],
    ],
    MIO_PAD_SLEEP_MODE_9 [
        OUT_9 OFFSET(0) NUMBITS(2) [],
    ],
    MIO_PAD_SLEEP_MODE_10 [
        OUT_10 OFFSET(0) NUMBITS(2) [],
    ],
    MIO_PAD_SLEEP_MODE_11 [
        OUT_11 OFFSET(0) NUMBITS(2) [],
    ],
    MIO_PAD_SLEEP_MODE_12 [
        OUT_12 OFFSET(0) NUMBITS(2) [],
    ],
    MIO_PAD_SLEEP_MODE_13 [
        OUT_13 OFFSET(0) NUMBITS(2) [],
    ],
    MIO_PAD_SLEEP_MODE_14 [
        OUT_14 OFFSET(0) NUMBITS(2) [],
    ],
    MIO_PAD_SLEEP_MODE_15 [
        OUT_15 OFFSET(0) NUMBITS(2) [],
    ],
    MIO_PAD_SLEEP_MODE_16 [
        OUT_16 OFFSET(0) NUMBITS(2) [],
    ],
    MIO_PAD_SLEEP_MODE_17 [
        OUT_17 OFFSET(0) NUMBITS(2) [],
    ],
    MIO_PAD_SLEEP_MODE_18 [
        OUT_18 OFFSET(0) NUMBITS(2) [],
    ],
    MIO_PAD_SLEEP_MODE_19 [
        OUT_19 OFFSET(0) NUMBITS(2) [],
    ],
    MIO_PAD_SLEEP_MODE_20 [
        OUT_20 OFFSET(0) NUMBITS(2) [],
    ],
    MIO_PAD_SLEEP_MODE_21 [
        OUT_21 OFFSET(0) NUMBITS(2) [],
    ],
    MIO_PAD_SLEEP_MODE_22 [
        OUT_22 OFFSET(0) NUMBITS(2) [],
    ],
    MIO_PAD_SLEEP_MODE_23 [
        OUT_23 OFFSET(0) NUMBITS(2) [],
    ],
    MIO_PAD_SLEEP_MODE_24 [
        OUT_24 OFFSET(0) NUMBITS(2) [],
    ],
    MIO_PAD_SLEEP_MODE_25 [
        OUT_25 OFFSET(0) NUMBITS(2) [],
    ],
    MIO_PAD_SLEEP_MODE_26 [
        OUT_26 OFFSET(0) NUMBITS(2) [],
    ],
    MIO_PAD_SLEEP_MODE_27 [
        OUT_27 OFFSET(0) NUMBITS(2) [],
    ],
    MIO_PAD_SLEEP_MODE_28 [
        OUT_28 OFFSET(0) NUMBITS(2) [],
    ],
    MIO_PAD_SLEEP_MODE_29 [
        OUT_29 OFFSET(0) NUMBITS(2) [],
    ],
    MIO_PAD_SLEEP_MODE_30 [
        OUT_30 OFFSET(0) NUMBITS(2) [],
    ],
    MIO_PAD_SLEEP_MODE_31 [
        OUT_31 OFFSET(0) NUMBITS(2) [],
    ],
    DIO_PAD_SLEEP_STATUS [
        EN_0 OFFSET(0) NUMBITS(1) [],
        EN_1 OFFSET(1) NUMBITS(1) [],
        EN_2 OFFSET(2) NUMBITS(1) [],
        EN_3 OFFSET(3) NUMBITS(1) [],
        EN_4 OFFSET(4) NUMBITS(1) [],
        EN_5 OFFSET(5) NUMBITS(1) [],
        EN_6 OFFSET(6) NUMBITS(1) [],
        EN_7 OFFSET(7) NUMBITS(1) [],
        EN_8 OFFSET(8) NUMBITS(1) [],
        EN_9 OFFSET(9) NUMBITS(1) [],
        EN_10 OFFSET(10) NUMBITS(1) [],
        EN_11 OFFSET(11) NUMBITS(1) [],
        EN_12 OFFSET(12) NUMBITS(1) [],
        EN_13 OFFSET(13) NUMBITS(1) [],
        EN_14 OFFSET(14) NUMBITS(1) [],
        EN_15 OFFSET(15) NUMBITS(1) [],
    ],
    DIO_PAD_SLEEP_REGWEN_0 [
        EN_0 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_SLEEP_REGWEN_1 [
        EN_1 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_SLEEP_REGWEN_2 [
        EN_2 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_SLEEP_REGWEN_3 [
        EN_3 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_SLEEP_REGWEN_4 [
        EN_4 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_SLEEP_REGWEN_5 [
        EN_5 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_SLEEP_REGWEN_6 [
        EN_6 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_SLEEP_REGWEN_7 [
        EN_7 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_SLEEP_REGWEN_8 [
        EN_8 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_SLEEP_REGWEN_9 [
        EN_9 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_SLEEP_REGWEN_10 [
        EN_10 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_SLEEP_REGWEN_11 [
        EN_11 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_SLEEP_REGWEN_12 [
        EN_12 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_SLEEP_REGWEN_13 [
        EN_13 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_SLEEP_REGWEN_14 [
        EN_14 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_SLEEP_REGWEN_15 [
        EN_15 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_SLEEP_EN_0 [
        EN_0 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_SLEEP_EN_1 [
        EN_1 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_SLEEP_EN_2 [
        EN_2 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_SLEEP_EN_3 [
        EN_3 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_SLEEP_EN_4 [
        EN_4 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_SLEEP_EN_5 [
        EN_5 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_SLEEP_EN_6 [
        EN_6 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_SLEEP_EN_7 [
        EN_7 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_SLEEP_EN_8 [
        EN_8 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_SLEEP_EN_9 [
        EN_9 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_SLEEP_EN_10 [
        EN_10 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_SLEEP_EN_11 [
        EN_11 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_SLEEP_EN_12 [
        EN_12 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_SLEEP_EN_13 [
        EN_13 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_SLEEP_EN_14 [
        EN_14 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_SLEEP_EN_15 [
        EN_15 OFFSET(0) NUMBITS(1) [],
    ],
    DIO_PAD_SLEEP_MODE_0 [
        OUT_0 OFFSET(0) NUMBITS(2) [
            TIE_LOW = 0,
            TIE_HIGH = 1,
            HIGH_Z = 2,
            KEEP = 3,
        ],
    ],
    DIO_PAD_SLEEP_MODE_1 [
        OUT_1 OFFSET(0) NUMBITS(2) [],
    ],
    DIO_PAD_SLEEP_MODE_2 [
        OUT_2 OFFSET(0) NUMBITS(2) [],
    ],
    DIO_PAD_SLEEP_MODE_3 [
        OUT_3 OFFSET(0) NUMBITS(2) [],
    ],
    DIO_PAD_SLEEP_MODE_4 [
        OUT_4 OFFSET(0) NUMBITS(2) [],
    ],
    DIO_PAD_SLEEP_MODE_5 [
        OUT_5 OFFSET(0) NUMBITS(2) [],
    ],
    DIO_PAD_SLEEP_MODE_6 [
        OUT_6 OFFSET(0) NUMBITS(2) [],
    ],
    DIO_PAD_SLEEP_MODE_7 [
        OUT_7 OFFSET(0) NUMBITS(2) [],
    ],
    DIO_PAD_SLEEP_MODE_8 [
        OUT_8 OFFSET(0) NUMBITS(2) [],
    ],
    DIO_PAD_SLEEP_MODE_9 [
        OUT_9 OFFSET(0) NUMBITS(2) [],
    ],
    DIO_PAD_SLEEP_MODE_10 [
        OUT_10 OFFSET(0) NUMBITS(2) [],
    ],
    DIO_PAD_SLEEP_MODE_11 [
        OUT_11 OFFSET(0) NUMBITS(2) [],
    ],
    DIO_PAD_SLEEP_MODE_12 [
        OUT_12 OFFSET(0) NUMBITS(2) [],
    ],
    DIO_PAD_SLEEP_MODE_13 [
        OUT_13 OFFSET(0) NUMBITS(2) [],
    ],
    DIO_PAD_SLEEP_MODE_14 [
        OUT_14 OFFSET(0) NUMBITS(2) [],
    ],
    DIO_PAD_SLEEP_MODE_15 [
        OUT_15 OFFSET(0) NUMBITS(2) [],
    ],
    WKUP_DETECTOR_REGWEN_0 [
        EN_0 OFFSET(0) NUMBITS(1) [],
    ],
    WKUP_DETECTOR_REGWEN_1 [
        EN_1 OFFSET(0) NUMBITS(1) [],
    ],
    WKUP_DETECTOR_REGWEN_2 [
        EN_2 OFFSET(0) NUMBITS(1) [],
    ],
    WKUP_DETECTOR_REGWEN_3 [
        EN_3 OFFSET(0) NUMBITS(1) [],
    ],
    WKUP_DETECTOR_REGWEN_4 [
        EN_4 OFFSET(0) NUMBITS(1) [],
    ],
    WKUP_DETECTOR_REGWEN_5 [
        EN_5 OFFSET(0) NUMBITS(1) [],
    ],
    WKUP_DETECTOR_REGWEN_6 [
        EN_6 OFFSET(0) NUMBITS(1) [],
    ],
    WKUP_DETECTOR_REGWEN_7 [
        EN_7 OFFSET(0) NUMBITS(1) [],
    ],
    WKUP_DETECTOR_EN_0 [
        EN_0 OFFSET(0) NUMBITS(1) [],
    ],
    WKUP_DETECTOR_EN_1 [
        EN_1 OFFSET(0) NUMBITS(1) [],
    ],
    WKUP_DETECTOR_EN_2 [
        EN_2 OFFSET(0) NUMBITS(1) [],
    ],
    WKUP_DETECTOR_EN_3 [
        EN_3 OFFSET(0) NUMBITS(1) [],
    ],
    WKUP_DETECTOR_EN_4 [
        EN_4 OFFSET(0) NUMBITS(1) [],
    ],
    WKUP_DETECTOR_EN_5 [
        EN_5 OFFSET(0) NUMBITS(1) [],
    ],
    WKUP_DETECTOR_EN_6 [
        EN_6 OFFSET(0) NUMBITS(1) [],
    ],
    WKUP_DETECTOR_EN_7 [
        EN_7 OFFSET(0) NUMBITS(1) [],
    ],
    WKUP_DETECTOR_0 [
        MODE_0 OFFSET(0) NUMBITS(3) [
            POSEDGE = 0,
            NEGEDGE = 1,
            EDGE = 2,
            TIMEDHIGH = 3,
            TIMEDLOW = 4,
        ],
        FILTER_0 OFFSET(3) NUMBITS(1) [],
        MIODIO_0 OFFSET(4) NUMBITS(1) [],
    ],
    WKUP_DETECTOR_1 [
        MODE_1 OFFSET(0) NUMBITS(3) [],
        FILTER_1 OFFSET(3) NUMBITS(1) [],
        MIODIO_1 OFFSET(4) NUMBITS(1) [],
    ],
    WKUP_DETECTOR_2 [
        MODE_2 OFFSET(0) NUMBITS(3) [],
        FILTER_2 OFFSET(3) NUMBITS(1) [],
        MIODIO_2 OFFSET(4) NUMBITS(1) [],
    ],
    WKUP_DETECTOR_3 [
        MODE_3 OFFSET(0) NUMBITS(3) [],
        FILTER_3 OFFSET(3) NUMBITS(1) [],
        MIODIO_3 OFFSET(4) NUMBITS(1) [],
    ],
    WKUP_DETECTOR_4 [
        MODE_4 OFFSET(0) NUMBITS(3) [],
        FILTER_4 OFFSET(3) NUMBITS(1) [],
        MIODIO_4 OFFSET(4) NUMBITS(1) [],
    ],
    WKUP_DETECTOR_5 [
        MODE_5 OFFSET(0) NUMBITS(3) [],
        FILTER_5 OFFSET(3) NUMBITS(1) [],
        MIODIO_5 OFFSET(4) NUMBITS(1) [],
    ],
    WKUP_DETECTOR_6 [
        MODE_6 OFFSET(0) NUMBITS(3) [],
        FILTER_6 OFFSET(3) NUMBITS(1) [],
        MIODIO_6 OFFSET(4) NUMBITS(1) [],
    ],
    WKUP_DETECTOR_7 [
        MODE_7 OFFSET(0) NUMBITS(3) [],
        FILTER_7 OFFSET(3) NUMBITS(1) [],
        MIODIO_7 OFFSET(4) NUMBITS(1) [],
    ],
    WKUP_DETECTOR_CNT_TH_0 [
        TH_0 OFFSET(0) NUMBITS(8) [],
    ],
    WKUP_DETECTOR_CNT_TH_1 [
        TH_1 OFFSET(0) NUMBITS(8) [],
    ],
    WKUP_DETECTOR_CNT_TH_2 [
        TH_2 OFFSET(0) NUMBITS(8) [],
    ],
    WKUP_DETECTOR_CNT_TH_3 [
        TH_3 OFFSET(0) NUMBITS(8) [],
    ],
    WKUP_DETECTOR_CNT_TH_4 [
        TH_4 OFFSET(0) NUMBITS(8) [],
    ],
    WKUP_DETECTOR_CNT_TH_5 [
        TH_5 OFFSET(0) NUMBITS(8) [],
    ],
    WKUP_DETECTOR_CNT_TH_6 [
        TH_6 OFFSET(0) NUMBITS(8) [],
    ],
    WKUP_DETECTOR_CNT_TH_7 [
        TH_7 OFFSET(0) NUMBITS(8) [],
    ],
    WKUP_DETECTOR_PADSEL_0 [
        SEL_0 OFFSET(0) NUMBITS(6) [],
    ],
    WKUP_DETECTOR_PADSEL_1 [
        SEL_1 OFFSET(0) NUMBITS(6) [],
    ],
    WKUP_DETECTOR_PADSEL_2 [
        SEL_2 OFFSET(0) NUMBITS(6) [],
    ],
    WKUP_DETECTOR_PADSEL_3 [
        SEL_3 OFFSET(0) NUMBITS(6) [],
    ],
    WKUP_DETECTOR_PADSEL_4 [
        SEL_4 OFFSET(0) NUMBITS(6) [],
    ],
    WKUP_DETECTOR_PADSEL_5 [
        SEL_5 OFFSET(0) NUMBITS(6) [],
    ],
    WKUP_DETECTOR_PADSEL_6 [
        SEL_6 OFFSET(0) NUMBITS(6) [],
    ],
    WKUP_DETECTOR_PADSEL_7 [
        SEL_7 OFFSET(0) NUMBITS(6) [],
    ],
    WKUP_CAUSE [
        CAUSE_0 OFFSET(0) NUMBITS(1) [],
        CAUSE_1 OFFSET(1) NUMBITS(1) [],
        CAUSE_2 OFFSET(2) NUMBITS(1) [],
        CAUSE_3 OFFSET(3) NUMBITS(1) [],
        CAUSE_4 OFFSET(4) NUMBITS(1) [],
        CAUSE_5 OFFSET(5) NUMBITS(1) [],
        CAUSE_6 OFFSET(6) NUMBITS(1) [],
        CAUSE_7 OFFSET(7) NUMBITS(1) [],
    ],
];

// Pad attribute data width
pub const PINMUX_PARAM_ATTR_DW: u32 = 10;

// Number of muxed peripheral inputs
pub const PINMUX_PARAM_N_MIO_PERIPH_IN: u32 = 33;

// Number of muxed peripheral outputs
pub const PINMUX_PARAM_N_MIO_PERIPH_OUT: u32 = 32;

// Number of muxed IO pads
pub const PINMUX_PARAM_N_MIO_PADS: u32 = 32;

// Number of dedicated IO pads
pub const PINMUX_PARAM_N_DIO_PADS: u32 = 16;

// Number of wakeup detectors
pub const PINMUX_PARAM_N_WKUP_DETECT: u32 = 8;

// Number of wakeup counter bits
pub const PINMUX_PARAM_WKUP_CNT_WIDTH: u32 = 8;

// Number of alerts
pub const PINMUX_PARAM_NUM_ALERTS: u32 = 1;

// Register width
pub const PINMUX_PARAM_REG_WIDTH: u32 = 32;

// Register write enable for MIO peripheral input selects. (common
// parameters)
pub const PINMUX_MIO_PERIPH_INSEL_REGWEN_EN_FIELD_WIDTH: u32 = 1;
pub const PINMUX_MIO_PERIPH_INSEL_REGWEN_EN_FIELDS_PER_REG: u32 = 32;
pub const PINMUX_MIO_PERIPH_INSEL_REGWEN_MULTIREG_COUNT: u32 = 33;

// For each peripheral input, this selects the muxable pad input. (common
// parameters)
pub const PINMUX_MIO_PERIPH_INSEL_IN_FIELD_WIDTH: u32 = 6;
pub const PINMUX_MIO_PERIPH_INSEL_IN_FIELDS_PER_REG: u32 = 5;
pub const PINMUX_MIO_PERIPH_INSEL_MULTIREG_COUNT: u32 = 33;

// Register write enable for MIO output selects. (common parameters)
pub const PINMUX_MIO_OUTSEL_REGWEN_EN_FIELD_WIDTH: u32 = 1;
pub const PINMUX_MIO_OUTSEL_REGWEN_EN_FIELDS_PER_REG: u32 = 32;
pub const PINMUX_MIO_OUTSEL_REGWEN_MULTIREG_COUNT: u32 = 32;

// For each muxable pad, this selects the peripheral output. (common
// parameters)
pub const PINMUX_MIO_OUTSEL_OUT_FIELD_WIDTH: u32 = 6;
pub const PINMUX_MIO_OUTSEL_OUT_FIELDS_PER_REG: u32 = 5;
pub const PINMUX_MIO_OUTSEL_MULTIREG_COUNT: u32 = 32;

// Register write enable for MIO PAD attributes. (common parameters)
pub const PINMUX_MIO_PAD_ATTR_REGWEN_EN_FIELD_WIDTH: u32 = 1;
pub const PINMUX_MIO_PAD_ATTR_REGWEN_EN_FIELDS_PER_REG: u32 = 32;
pub const PINMUX_MIO_PAD_ATTR_REGWEN_MULTIREG_COUNT: u32 = 32;

// Muxed pad attributes.
pub const PINMUX_MIO_PAD_ATTR_ATTR_FIELD_WIDTH: u32 = 13;
pub const PINMUX_MIO_PAD_ATTR_ATTR_FIELDS_PER_REG: u32 = 2;
pub const PINMUX_MIO_PAD_ATTR_MULTIREG_COUNT: u32 = 32;

// Register write enable for DIO PAD attributes. (common parameters)
pub const PINMUX_DIO_PAD_ATTR_REGWEN_EN_FIELD_WIDTH: u32 = 1;
pub const PINMUX_DIO_PAD_ATTR_REGWEN_EN_FIELDS_PER_REG: u32 = 32;
pub const PINMUX_DIO_PAD_ATTR_REGWEN_MULTIREG_COUNT: u32 = 16;

// Dedicated pad attributes.
pub const PINMUX_DIO_PAD_ATTR_ATTR_FIELD_WIDTH: u32 = 13;
pub const PINMUX_DIO_PAD_ATTR_ATTR_FIELDS_PER_REG: u32 = 2;
pub const PINMUX_DIO_PAD_ATTR_MULTIREG_COUNT: u32 = 16;

// Register indicating whether the corresponding pad is in sleep mode.
// (common parameters)
pub const PINMUX_MIO_PAD_SLEEP_STATUS_EN_FIELD_WIDTH: u32 = 1;
pub const PINMUX_MIO_PAD_SLEEP_STATUS_EN_FIELDS_PER_REG: u32 = 32;
pub const PINMUX_MIO_PAD_SLEEP_STATUS_MULTIREG_COUNT: u32 = 1;

// Register write enable for MIO sleep value configuration. (common
// parameters)
pub const PINMUX_MIO_PAD_SLEEP_REGWEN_EN_FIELD_WIDTH: u32 = 1;
pub const PINMUX_MIO_PAD_SLEEP_REGWEN_EN_FIELDS_PER_REG: u32 = 32;
pub const PINMUX_MIO_PAD_SLEEP_REGWEN_MULTIREG_COUNT: u32 = 32;

// Enables the sleep mode of the corresponding muxed pad. (common parameters)
pub const PINMUX_MIO_PAD_SLEEP_EN_EN_FIELD_WIDTH: u32 = 1;
pub const PINMUX_MIO_PAD_SLEEP_EN_EN_FIELDS_PER_REG: u32 = 32;
pub const PINMUX_MIO_PAD_SLEEP_EN_MULTIREG_COUNT: u32 = 32;

// Defines sleep behavior of the corresponding muxed pad. (common parameters)
pub const PINMUX_MIO_PAD_SLEEP_MODE_OUT_FIELD_WIDTH: u32 = 2;
pub const PINMUX_MIO_PAD_SLEEP_MODE_OUT_FIELDS_PER_REG: u32 = 16;
pub const PINMUX_MIO_PAD_SLEEP_MODE_MULTIREG_COUNT: u32 = 32;

// Register indicating whether the corresponding pad is in sleep mode.
// (common parameters)
pub const PINMUX_DIO_PAD_SLEEP_STATUS_EN_FIELD_WIDTH: u32 = 1;
pub const PINMUX_DIO_PAD_SLEEP_STATUS_EN_FIELDS_PER_REG: u32 = 32;
pub const PINMUX_DIO_PAD_SLEEP_STATUS_MULTIREG_COUNT: u32 = 1;

// Register write enable for DIO sleep value configuration. (common
// parameters)
pub const PINMUX_DIO_PAD_SLEEP_REGWEN_EN_FIELD_WIDTH: u32 = 1;
pub const PINMUX_DIO_PAD_SLEEP_REGWEN_EN_FIELDS_PER_REG: u32 = 32;
pub const PINMUX_DIO_PAD_SLEEP_REGWEN_MULTIREG_COUNT: u32 = 16;

// Enables the sleep mode of the corresponding dedicated pad. (common
// parameters)
pub const PINMUX_DIO_PAD_SLEEP_EN_EN_FIELD_WIDTH: u32 = 1;
pub const PINMUX_DIO_PAD_SLEEP_EN_EN_FIELDS_PER_REG: u32 = 32;
pub const PINMUX_DIO_PAD_SLEEP_EN_MULTIREG_COUNT: u32 = 16;

// Defines sleep behavior of the corresponding dedicated pad. (common
// parameters)
pub const PINMUX_DIO_PAD_SLEEP_MODE_OUT_FIELD_WIDTH: u32 = 2;
pub const PINMUX_DIO_PAD_SLEEP_MODE_OUT_FIELDS_PER_REG: u32 = 16;
pub const PINMUX_DIO_PAD_SLEEP_MODE_MULTIREG_COUNT: u32 = 16;

// Register write enable for wakeup detectors. (common parameters)
pub const PINMUX_WKUP_DETECTOR_REGWEN_EN_FIELD_WIDTH: u32 = 1;
pub const PINMUX_WKUP_DETECTOR_REGWEN_EN_FIELDS_PER_REG: u32 = 32;
pub const PINMUX_WKUP_DETECTOR_REGWEN_MULTIREG_COUNT: u32 = 8;

// Enables for the wakeup detectors.
pub const PINMUX_WKUP_DETECTOR_EN_EN_FIELD_WIDTH: u32 = 1;
pub const PINMUX_WKUP_DETECTOR_EN_EN_FIELDS_PER_REG: u32 = 32;
pub const PINMUX_WKUP_DETECTOR_EN_MULTIREG_COUNT: u32 = 8;

// Counter thresholds for wakeup condition detectors.
pub const PINMUX_WKUP_DETECTOR_CNT_TH_TH_FIELD_WIDTH: u32 = 8;
pub const PINMUX_WKUP_DETECTOR_CNT_TH_TH_FIELDS_PER_REG: u32 = 4;
pub const PINMUX_WKUP_DETECTOR_CNT_TH_MULTIREG_COUNT: u32 = 8;

// Pad selects for pad wakeup condition detectors.
pub const PINMUX_WKUP_DETECTOR_PADSEL_SEL_FIELD_WIDTH: u32 = 6;
pub const PINMUX_WKUP_DETECTOR_PADSEL_SEL_FIELDS_PER_REG: u32 = 5;
pub const PINMUX_WKUP_DETECTOR_PADSEL_MULTIREG_COUNT: u32 = 8;

// Cause registers for wakeup detectors.
pub const PINMUX_WKUP_CAUSE_CAUSE_FIELD_WIDTH: u32 = 1;
pub const PINMUX_WKUP_CAUSE_CAUSE_FIELDS_PER_REG: u32 = 32;
pub const PINMUX_WKUP_CAUSE_MULTIREG_COUNT: u32 = 1;

// End generated register constants for PINMUX

