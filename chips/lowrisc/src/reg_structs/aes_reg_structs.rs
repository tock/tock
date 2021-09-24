// Generated register struct for aes

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
    pub AesRegisters {
        (0x0 => alert_test: WriteOnly<u32, ALERT_TEST::Register>),
        (0x4 => key_share0_0: WriteOnly<u32, KEY_SHARE0_0::Register>),
        (0x8 => key_share0_1: WriteOnly<u32, KEY_SHARE0_1::Register>),
        (0xc => key_share0_2: WriteOnly<u32, KEY_SHARE0_2::Register>),
        (0x10 => key_share0_3: WriteOnly<u32, KEY_SHARE0_3::Register>),
        (0x14 => key_share0_4: WriteOnly<u32, KEY_SHARE0_4::Register>),
        (0x18 => key_share0_5: WriteOnly<u32, KEY_SHARE0_5::Register>),
        (0x1c => key_share0_6: WriteOnly<u32, KEY_SHARE0_6::Register>),
        (0x20 => key_share0_7: WriteOnly<u32, KEY_SHARE0_7::Register>),
        (0x24 => key_share1_0: WriteOnly<u32, KEY_SHARE1_0::Register>),
        (0x28 => key_share1_1: WriteOnly<u32, KEY_SHARE1_1::Register>),
        (0x2c => key_share1_2: WriteOnly<u32, KEY_SHARE1_2::Register>),
        (0x30 => key_share1_3: WriteOnly<u32, KEY_SHARE1_3::Register>),
        (0x34 => key_share1_4: WriteOnly<u32, KEY_SHARE1_4::Register>),
        (0x38 => key_share1_5: WriteOnly<u32, KEY_SHARE1_5::Register>),
        (0x3c => key_share1_6: WriteOnly<u32, KEY_SHARE1_6::Register>),
        (0x40 => key_share1_7: WriteOnly<u32, KEY_SHARE1_7::Register>),
        (0x44 => iv_0: WriteOnly<u32, IV_0::Register>),
        (0x48 => iv_1: WriteOnly<u32, IV_1::Register>),
        (0x4c => iv_2: WriteOnly<u32, IV_2::Register>),
        (0x50 => iv_3: WriteOnly<u32, IV_3::Register>),
        (0x54 => data_in_0: WriteOnly<u32, DATA_IN_0::Register>),
        (0x58 => data_in_1: WriteOnly<u32, DATA_IN_1::Register>),
        (0x5c => data_in_2: WriteOnly<u32, DATA_IN_2::Register>),
        (0x60 => data_in_3: WriteOnly<u32, DATA_IN_3::Register>),
        (0x64 => data_out_0: ReadOnly<u32, DATA_OUT_0::Register>),
        (0x68 => data_out_1: ReadOnly<u32, DATA_OUT_1::Register>),
        (0x6c => data_out_2: ReadOnly<u32, DATA_OUT_2::Register>),
        (0x70 => data_out_3: ReadOnly<u32, DATA_OUT_3::Register>),
        (0x74 => ctrl_shadowed: ReadWrite<u32, CTRL_SHADOWED::Register>),
        (0x78 => trigger: WriteOnly<u32, TRIGGER::Register>),
        (0x7c => status: ReadOnly<u32, STATUS::Register>),
    }
}

register_bitfields![u32,
    ALERT_TEST [
        RECOV_CTRL_UPDATE_ERR OFFSET(0) NUMBITS(1) [],
        FATAL_FAULT OFFSET(1) NUMBITS(1) [],
    ],
    KEY_SHARE0_0 [
        KEY_SHARE0_0 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE0_1 [
        KEY_SHARE0_1 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE0_2 [
        KEY_SHARE0_2 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE0_3 [
        KEY_SHARE0_3 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE0_4 [
        KEY_SHARE0_4 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE0_5 [
        KEY_SHARE0_5 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE0_6 [
        KEY_SHARE0_6 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE0_7 [
        KEY_SHARE0_7 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE1_0 [
        KEY_SHARE1_0 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE1_1 [
        KEY_SHARE1_1 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE1_2 [
        KEY_SHARE1_2 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE1_3 [
        KEY_SHARE1_3 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE1_4 [
        KEY_SHARE1_4 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE1_5 [
        KEY_SHARE1_5 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE1_6 [
        KEY_SHARE1_6 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE1_7 [
        KEY_SHARE1_7 OFFSET(0) NUMBITS(32) [],
    ],
    IV_0 [
        IV_0 OFFSET(0) NUMBITS(32) [],
    ],
    IV_1 [
        IV_1 OFFSET(0) NUMBITS(32) [],
    ],
    IV_2 [
        IV_2 OFFSET(0) NUMBITS(32) [],
    ],
    IV_3 [
        IV_3 OFFSET(0) NUMBITS(32) [],
    ],
    DATA_IN_0 [
        DATA_IN_0 OFFSET(0) NUMBITS(32) [],
    ],
    DATA_IN_1 [
        DATA_IN_1 OFFSET(0) NUMBITS(32) [],
    ],
    DATA_IN_2 [
        DATA_IN_2 OFFSET(0) NUMBITS(32) [],
    ],
    DATA_IN_3 [
        DATA_IN_3 OFFSET(0) NUMBITS(32) [],
    ],
    DATA_OUT_0 [
        DATA_OUT_0 OFFSET(0) NUMBITS(32) [],
    ],
    DATA_OUT_1 [
        DATA_OUT_1 OFFSET(0) NUMBITS(32) [],
    ],
    DATA_OUT_2 [
        DATA_OUT_2 OFFSET(0) NUMBITS(32) [],
    ],
    DATA_OUT_3 [
        DATA_OUT_3 OFFSET(0) NUMBITS(32) [],
    ],
    CTRL_SHADOWED [
        OPERATION OFFSET(0) NUMBITS(1) [],
        MODE OFFSET(1) NUMBITS(6) [
            AES_ECB = 1,
            AES_CBC = 2,
            AES_CFB = 4,
            AES_OFB = 8,
            AES_CTR = 16,
            AES_NONE = 32,
        ],
        KEY_LEN OFFSET(7) NUMBITS(3) [
            AES_128 = 1,
            AES_192 = 2,
            AES_256 = 4,
        ],
        SIDELOAD OFFSET(10) NUMBITS(1) [],
        MANUAL_OPERATION OFFSET(11) NUMBITS(1) [],
        FORCE_ZERO_MASKS OFFSET(12) NUMBITS(1) [],
    ],
    TRIGGER [
        START OFFSET(0) NUMBITS(1) [],
        KEY_IV_DATA_IN_CLEAR OFFSET(1) NUMBITS(1) [],
        DATA_OUT_CLEAR OFFSET(2) NUMBITS(1) [],
        PRNG_RESEED OFFSET(3) NUMBITS(1) [],
    ],
    STATUS [
        IDLE OFFSET(0) NUMBITS(1) [],
        STALL OFFSET(1) NUMBITS(1) [],
        OUTPUT_LOST OFFSET(2) NUMBITS(1) [],
        OUTPUT_VALID OFFSET(3) NUMBITS(1) [],
        INPUT_READY OFFSET(4) NUMBITS(1) [],
        ALERT_RECOV_CTRL_UPDATE_ERR OFFSET(5) NUMBITS(1) [],
        ALERT_FATAL_FAULT OFFSET(6) NUMBITS(1) [],
    ],
];

// Number registers for key
pub const AES_PARAM_NUM_REGS_KEY: u32 = 8;

// Number registers for initialization vector
pub const AES_PARAM_NUM_REGS_IV: u32 = 4;

// Number registers for input and output data
pub const AES_PARAM_NUM_REGS_DATA: u32 = 4;

// Number of alerts
pub const AES_PARAM_NUM_ALERTS: u32 = 2;

// Register width
pub const AES_PARAM_REG_WIDTH: u32 = 32;

// Initial Key Registers Share 0.
pub const AES_KEY_SHARE0_KEY_SHARE0_FIELD_WIDTH: u32 = 32;
pub const AES_KEY_SHARE0_KEY_SHARE0_FIELDS_PER_REG: u32 = 1;
pub const AES_KEY_SHARE0_MULTIREG_COUNT: u32 = 8;

// Initial Key Registers Share 1.
pub const AES_KEY_SHARE1_KEY_SHARE1_FIELD_WIDTH: u32 = 32;
pub const AES_KEY_SHARE1_KEY_SHARE1_FIELDS_PER_REG: u32 = 1;
pub const AES_KEY_SHARE1_MULTIREG_COUNT: u32 = 8;

// Initialization Vector Registers.
pub const AES_IV_IV_FIELD_WIDTH: u32 = 32;
pub const AES_IV_IV_FIELDS_PER_REG: u32 = 1;
pub const AES_IV_MULTIREG_COUNT: u32 = 4;

// Input Data Registers.
pub const AES_DATA_IN_DATA_IN_FIELD_WIDTH: u32 = 32;
pub const AES_DATA_IN_DATA_IN_FIELDS_PER_REG: u32 = 1;
pub const AES_DATA_IN_MULTIREG_COUNT: u32 = 4;

// Output Data Register.
pub const AES_DATA_OUT_DATA_OUT_FIELD_WIDTH: u32 = 32;
pub const AES_DATA_OUT_DATA_OUT_FIELDS_PER_REG: u32 = 1;
pub const AES_DATA_OUT_MULTIREG_COUNT: u32 = 4;

// End generated register constants for aes

