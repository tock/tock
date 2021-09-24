// Generated register struct for rom_ctrl

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
    pub Rom_CtrlRegisters {
        (0x0 => alert_test: WriteOnly<u32, ALERT_TEST::Register>),
        (0x4 => fatal_alert_cause: ReadOnly<u32, FATAL_ALERT_CAUSE::Register>),
        (0x8 => digest_0: ReadOnly<u32, DIGEST_0::Register>),
        (0xc => digest_1: ReadOnly<u32, DIGEST_1::Register>),
        (0x10 => digest_2: ReadOnly<u32, DIGEST_2::Register>),
        (0x14 => digest_3: ReadOnly<u32, DIGEST_3::Register>),
        (0x18 => digest_4: ReadOnly<u32, DIGEST_4::Register>),
        (0x1c => digest_5: ReadOnly<u32, DIGEST_5::Register>),
        (0x20 => digest_6: ReadOnly<u32, DIGEST_6::Register>),
        (0x24 => digest_7: ReadOnly<u32, DIGEST_7::Register>),
        (0x28 => exp_digest_0: ReadOnly<u32, EXP_DIGEST_0::Register>),
        (0x2c => exp_digest_1: ReadOnly<u32, EXP_DIGEST_1::Register>),
        (0x30 => exp_digest_2: ReadOnly<u32, EXP_DIGEST_2::Register>),
        (0x34 => exp_digest_3: ReadOnly<u32, EXP_DIGEST_3::Register>),
        (0x38 => exp_digest_4: ReadOnly<u32, EXP_DIGEST_4::Register>),
        (0x3c => exp_digest_5: ReadOnly<u32, EXP_DIGEST_5::Register>),
        (0x40 => exp_digest_6: ReadOnly<u32, EXP_DIGEST_6::Register>),
        (0x44 => exp_digest_7: ReadOnly<u32, EXP_DIGEST_7::Register>),
    }
}

register_bitfields![u32,
    ALERT_TEST [
        FATAL OFFSET(0) NUMBITS(1) [],
    ],
    FATAL_ALERT_CAUSE [
        CHECKER_ERROR OFFSET(0) NUMBITS(1) [],
        INTEGRITY_ERROR OFFSET(1) NUMBITS(1) [],
    ],
    DIGEST_0 [
        DIGEST_0 OFFSET(0) NUMBITS(32) [],
    ],
    DIGEST_1 [
        DIGEST_1 OFFSET(0) NUMBITS(32) [],
    ],
    DIGEST_2 [
        DIGEST_2 OFFSET(0) NUMBITS(32) [],
    ],
    DIGEST_3 [
        DIGEST_3 OFFSET(0) NUMBITS(32) [],
    ],
    DIGEST_4 [
        DIGEST_4 OFFSET(0) NUMBITS(32) [],
    ],
    DIGEST_5 [
        DIGEST_5 OFFSET(0) NUMBITS(32) [],
    ],
    DIGEST_6 [
        DIGEST_6 OFFSET(0) NUMBITS(32) [],
    ],
    DIGEST_7 [
        DIGEST_7 OFFSET(0) NUMBITS(32) [],
    ],
    EXP_DIGEST_0 [
        DIGEST_0 OFFSET(0) NUMBITS(32) [],
    ],
    EXP_DIGEST_1 [
        DIGEST_1 OFFSET(0) NUMBITS(32) [],
    ],
    EXP_DIGEST_2 [
        DIGEST_2 OFFSET(0) NUMBITS(32) [],
    ],
    EXP_DIGEST_3 [
        DIGEST_3 OFFSET(0) NUMBITS(32) [],
    ],
    EXP_DIGEST_4 [
        DIGEST_4 OFFSET(0) NUMBITS(32) [],
    ],
    EXP_DIGEST_5 [
        DIGEST_5 OFFSET(0) NUMBITS(32) [],
    ],
    EXP_DIGEST_6 [
        DIGEST_6 OFFSET(0) NUMBITS(32) [],
    ],
    EXP_DIGEST_7 [
        DIGEST_7 OFFSET(0) NUMBITS(32) [],
    ],
];

// Number of alerts
pub const ROM_CTRL_PARAM_NUM_ALERTS: u32 = 1;

// Register width
pub const ROM_CTRL_PARAM_REG_WIDTH: u32 = 32;

// The digest computed from the contents of ROM (common parameters)
pub const ROM_CTRL_DIGEST_DIGEST_FIELD_WIDTH: u32 = 32;
pub const ROM_CTRL_DIGEST_DIGEST_FIELDS_PER_REG: u32 = 1;
pub const ROM_CTRL_DIGEST_MULTIREG_COUNT: u32 = 8;

// The expected digest, stored in the top words of ROM (common parameters)
pub const ROM_CTRL_EXP_DIGEST_DIGEST_FIELD_WIDTH: u32 = 32;
pub const ROM_CTRL_EXP_DIGEST_DIGEST_FIELDS_PER_REG: u32 = 1;
pub const ROM_CTRL_EXP_DIGEST_MULTIREG_COUNT: u32 = 8;

// Memory area: ROM data
pub const ROM_CTRL_ROM_REG_OFFSET: usize = 0x0;
pub const ROM_CTRL_ROM_SIZE_WORDS: u32 = 4096;
pub const ROM_CTRL_ROM_SIZE_BYTES: u32 = 16384;
// End generated register constants for rom_ctrl

