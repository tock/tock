// Generated register struct for lc_ctrl

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
    pub Lc_CtrlRegisters {
        (0x0 => alert_test: WriteOnly<u32, ALERT_TEST::Register>),
        (0x4 => status: ReadOnly<u32, STATUS::Register>),
        (0x8 => claim_transition_if: ReadWrite<u32, CLAIM_TRANSITION_IF::Register>),
        (0xc => transition_regwen: ReadOnly<u32, TRANSITION_REGWEN::Register>),
        (0x10 => transition_cmd: WriteOnly<u32, TRANSITION_CMD::Register>),
        (0x14 => transition_ctrl: ReadWrite<u32, TRANSITION_CTRL::Register>),
        (0x18 => transition_token_0: ReadWrite<u32, TRANSITION_TOKEN_0::Register>),
        (0x1c => transition_token_1: ReadWrite<u32, TRANSITION_TOKEN_1::Register>),
        (0x20 => transition_token_2: ReadWrite<u32, TRANSITION_TOKEN_2::Register>),
        (0x24 => transition_token_3: ReadWrite<u32, TRANSITION_TOKEN_3::Register>),
        (0x28 => transition_target: ReadWrite<u32, TRANSITION_TARGET::Register>),
        (0x2c => otp_vendor_test_ctrl: ReadWrite<u32, OTP_VENDOR_TEST_CTRL::Register>),
        (0x30 => otp_vendor_test_status: ReadOnly<u32, OTP_VENDOR_TEST_STATUS::Register>),
        (0x34 => lc_state: ReadOnly<u32, LC_STATE::Register>),
        (0x38 => lc_transition_cnt: ReadOnly<u32, LC_TRANSITION_CNT::Register>),
        (0x3c => lc_id_state: ReadOnly<u32, LC_ID_STATE::Register>),
        (0x40 => device_id_0: ReadOnly<u32, DEVICE_ID_0::Register>),
        (0x44 => device_id_1: ReadOnly<u32, DEVICE_ID_1::Register>),
        (0x48 => device_id_2: ReadOnly<u32, DEVICE_ID_2::Register>),
        (0x4c => device_id_3: ReadOnly<u32, DEVICE_ID_3::Register>),
        (0x50 => device_id_4: ReadOnly<u32, DEVICE_ID_4::Register>),
        (0x54 => device_id_5: ReadOnly<u32, DEVICE_ID_5::Register>),
        (0x58 => device_id_6: ReadOnly<u32, DEVICE_ID_6::Register>),
        (0x5c => device_id_7: ReadOnly<u32, DEVICE_ID_7::Register>),
        (0x60 => manuf_state_0: ReadOnly<u32, MANUF_STATE_0::Register>),
        (0x64 => manuf_state_1: ReadOnly<u32, MANUF_STATE_1::Register>),
        (0x68 => manuf_state_2: ReadOnly<u32, MANUF_STATE_2::Register>),
        (0x6c => manuf_state_3: ReadOnly<u32, MANUF_STATE_3::Register>),
        (0x70 => manuf_state_4: ReadOnly<u32, MANUF_STATE_4::Register>),
        (0x74 => manuf_state_5: ReadOnly<u32, MANUF_STATE_5::Register>),
        (0x78 => manuf_state_6: ReadOnly<u32, MANUF_STATE_6::Register>),
        (0x7c => manuf_state_7: ReadOnly<u32, MANUF_STATE_7::Register>),
    }
}

register_bitfields![u32,
    ALERT_TEST [
        FATAL_PROG_ERROR OFFSET(0) NUMBITS(1) [],
        FATAL_STATE_ERROR OFFSET(1) NUMBITS(1) [],
        FATAL_BUS_INTEG_ERROR OFFSET(2) NUMBITS(1) [],
    ],
    STATUS [
        READY OFFSET(0) NUMBITS(1) [],
        TRANSITION_SUCCESSFUL OFFSET(1) NUMBITS(1) [],
        TRANSITION_COUNT_ERROR OFFSET(2) NUMBITS(1) [],
        TRANSITION_ERROR OFFSET(3) NUMBITS(1) [],
        TOKEN_ERROR OFFSET(4) NUMBITS(1) [],
        FLASH_RMA_ERROR OFFSET(5) NUMBITS(1) [],
        OTP_ERROR OFFSET(6) NUMBITS(1) [],
        STATE_ERROR OFFSET(7) NUMBITS(1) [],
        BUS_INTEG_ERROR OFFSET(8) NUMBITS(1) [],
        OTP_PARTITION_ERROR OFFSET(9) NUMBITS(1) [],
    ],
    CLAIM_TRANSITION_IF [
        MUTEX OFFSET(0) NUMBITS(8) [],
    ],
    TRANSITION_REGWEN [
        TRANSITION_REGWEN OFFSET(0) NUMBITS(1) [],
    ],
    TRANSITION_CMD [
        START OFFSET(0) NUMBITS(1) [],
    ],
    TRANSITION_CTRL [
        EXT_CLOCK_EN OFFSET(0) NUMBITS(1) [],
    ],
    TRANSITION_TOKEN_0 [
        TRANSITION_TOKEN_0 OFFSET(0) NUMBITS(32) [],
    ],
    TRANSITION_TOKEN_1 [
        TRANSITION_TOKEN_1 OFFSET(0) NUMBITS(32) [],
    ],
    TRANSITION_TOKEN_2 [
        TRANSITION_TOKEN_2 OFFSET(0) NUMBITS(32) [],
    ],
    TRANSITION_TOKEN_3 [
        TRANSITION_TOKEN_3 OFFSET(0) NUMBITS(32) [],
    ],
    TRANSITION_TARGET [
        STATE OFFSET(0) NUMBITS(5) [
            RAW = 0,
            TEST_UNLOCKED0 = 1,
            TEST_LOCKED0 = 2,
            TEST_UNLOCKED1 = 3,
            TEST_LOCKED1 = 4,
            TEST_UNLOCKED2 = 5,
            TEST_LOCKED2 = 6,
            TEST_UNLOCKED3 = 7,
            TEST_LOCKED3 = 8,
            TEST_UNLOCKED4 = 9,
            TEST_LOCKED4 = 10,
            TEST_UNLOCKED5 = 11,
            TEST_LOCKED5 = 12,
            TEST_UNLOCKED6 = 13,
            TEST_LOCKED6 = 14,
            TEST_UNLOCKED7 = 15,
            DEV = 16,
            PROD = 17,
            PROD_END = 18,
            RMA = 19,
            SCRAP = 20,
        ],
    ],
    OTP_VENDOR_TEST_CTRL [
        OTP_VENDOR_TEST_CTRL OFFSET(0) NUMBITS(32) [],
    ],
    OTP_VENDOR_TEST_STATUS [
        OTP_VENDOR_TEST_STATUS OFFSET(0) NUMBITS(32) [],
    ],
    LC_STATE [
        STATE OFFSET(0) NUMBITS(5) [
            RAW = 0,
            TEST_UNLOCKED0 = 1,
            TEST_LOCKED0 = 2,
            TEST_UNLOCKED1 = 3,
            TEST_LOCKED1 = 4,
            TEST_UNLOCKED2 = 5,
            TEST_LOCKED2 = 6,
            TEST_UNLOCKED3 = 7,
            TEST_LOCKED3 = 8,
            TEST_UNLOCKED4 = 9,
            TEST_LOCKED4 = 10,
            TEST_UNLOCKED5 = 11,
            TEST_LOCKED5 = 12,
            TEST_UNLOCKED6 = 13,
            TEST_LOCKED6 = 14,
            TEST_UNLOCKED7 = 15,
            DEV = 16,
            PROD = 17,
            PROD_END = 18,
            RMA = 19,
            SCRAP = 20,
            POST_TRANSITION = 21,
            ESCALATE = 22,
            INVALID = 23,
        ],
    ],
    LC_TRANSITION_CNT [
        CNT OFFSET(0) NUMBITS(5) [],
    ],
    LC_ID_STATE [
        STATE OFFSET(0) NUMBITS(2) [
            BLANK = 0,
            PERSONALIZED = 1,
            INVALID = 2,
        ],
    ],
    DEVICE_ID_0 [
        DEVICE_ID_0 OFFSET(0) NUMBITS(32) [],
    ],
    DEVICE_ID_1 [
        DEVICE_ID_1 OFFSET(0) NUMBITS(32) [],
    ],
    DEVICE_ID_2 [
        DEVICE_ID_2 OFFSET(0) NUMBITS(32) [],
    ],
    DEVICE_ID_3 [
        DEVICE_ID_3 OFFSET(0) NUMBITS(32) [],
    ],
    DEVICE_ID_4 [
        DEVICE_ID_4 OFFSET(0) NUMBITS(32) [],
    ],
    DEVICE_ID_5 [
        DEVICE_ID_5 OFFSET(0) NUMBITS(32) [],
    ],
    DEVICE_ID_6 [
        DEVICE_ID_6 OFFSET(0) NUMBITS(32) [],
    ],
    DEVICE_ID_7 [
        DEVICE_ID_7 OFFSET(0) NUMBITS(32) [],
    ],
    MANUF_STATE_0 [
        MANUF_STATE_0 OFFSET(0) NUMBITS(32) [],
    ],
    MANUF_STATE_1 [
        MANUF_STATE_1 OFFSET(0) NUMBITS(32) [],
    ],
    MANUF_STATE_2 [
        MANUF_STATE_2 OFFSET(0) NUMBITS(32) [],
    ],
    MANUF_STATE_3 [
        MANUF_STATE_3 OFFSET(0) NUMBITS(32) [],
    ],
    MANUF_STATE_4 [
        MANUF_STATE_4 OFFSET(0) NUMBITS(32) [],
    ],
    MANUF_STATE_5 [
        MANUF_STATE_5 OFFSET(0) NUMBITS(32) [],
    ],
    MANUF_STATE_6 [
        MANUF_STATE_6 OFFSET(0) NUMBITS(32) [],
    ],
    MANUF_STATE_7 [
        MANUF_STATE_7 OFFSET(0) NUMBITS(32) [],
    ],
];

// Number of 32bit words in a token.
pub const LC_CTRL_PARAM_NUM_TOKEN_WORDS: u32 = 4;

// Number of life cycle state enum bits.
pub const LC_CTRL_PARAM_CSR_LC_STATE_WIDTH: u32 = 5;

// Number of life cycle transition counter bits.
pub const LC_CTRL_PARAM_CSR_LC_COUNT_WIDTH: u32 = 5;

// Number of life cycle id state enum bits.
pub const LC_CTRL_PARAM_CSR_LC_ID_STATE_WIDTH: u32 = 2;

// Number of vendor/test-specific OTP control bits.
pub const LC_CTRL_PARAM_CSR_OTP_TEST_CTRL_WIDTH: u32 = 32;

// Number of vendor/test-specific OTP status bits.
pub const LC_CTRL_PARAM_CSR_OTP_TEST_STATUS_WIDTH: u32 = 32;

// Number of 32bit words in the Device ID.
pub const LC_CTRL_PARAM_NUM_DEVICE_ID_WORDS: u32 = 8;

// Number of 32bit words in the manufacturing state.
pub const LC_CTRL_PARAM_NUM_MANUF_STATE_WORDS: u32 = 8;

// Number of alerts
pub const LC_CTRL_PARAM_NUM_ALERTS: u32 = 3;

// Register width
pub const LC_CTRL_PARAM_REG_WIDTH: u32 = 32;

// 128bit token for conditional transitions.
pub const LC_CTRL_TRANSITION_TOKEN_TRANSITION_TOKEN_FIELD_WIDTH: u32 = 32;
pub const LC_CTRL_TRANSITION_TOKEN_TRANSITION_TOKEN_FIELDS_PER_REG: u32 = 1;
pub const LC_CTRL_TRANSITION_TOKEN_MULTIREG_COUNT: u32 = 4;

// This is the 256bit DEVICE_ID value that is stored in the HW_CFG partition
// in OTP.
pub const LC_CTRL_DEVICE_ID_DEVICE_ID_FIELD_WIDTH: u32 = 32;
pub const LC_CTRL_DEVICE_ID_DEVICE_ID_FIELDS_PER_REG: u32 = 1;
pub const LC_CTRL_DEVICE_ID_MULTIREG_COUNT: u32 = 8;

// This is a 256bit field used for keeping track of the manufacturing state.
// (common parameters)
pub const LC_CTRL_MANUF_STATE_MANUF_STATE_FIELD_WIDTH: u32 = 32;
pub const LC_CTRL_MANUF_STATE_MANUF_STATE_FIELDS_PER_REG: u32 = 1;
pub const LC_CTRL_MANUF_STATE_MULTIREG_COUNT: u32 = 8;

// End generated register constants for lc_ctrl

