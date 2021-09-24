// Generated register struct for kmac

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
    pub KmacRegisters {
        (0x0 => intr_state: ReadWrite<u32, INTR_STATE::Register>),
        (0x4 => intr_enable: ReadWrite<u32, INTR_ENABLE::Register>),
        (0x8 => intr_test: WriteOnly<u32, INTR_TEST::Register>),
        (0xc => alert_test: WriteOnly<u32, ALERT_TEST::Register>),
        (0x10 => cfg_regwen: ReadOnly<u32, CFG_REGWEN::Register>),
        (0x14 => cfg: ReadWrite<u32, CFG::Register>),
        (0x18 => cmd: WriteOnly<u32, CMD::Register>),
        (0x1c => status: ReadOnly<u32, STATUS::Register>),
        (0x20 => entropy_period: ReadWrite<u32, ENTROPY_PERIOD::Register>),
        (0x24 => entropy_refresh: ReadWrite<u32, ENTROPY_REFRESH::Register>),
        (0x28 => entropy_seed_lower: ReadWrite<u32, ENTROPY_SEED_LOWER::Register>),
        (0x2c => entropy_seed_upper: ReadWrite<u32, ENTROPY_SEED_UPPER::Register>),
        (0x30 => key_share0_0: WriteOnly<u32, KEY_SHARE0_0::Register>),
        (0x34 => key_share0_1: WriteOnly<u32, KEY_SHARE0_1::Register>),
        (0x38 => key_share0_2: WriteOnly<u32, KEY_SHARE0_2::Register>),
        (0x3c => key_share0_3: WriteOnly<u32, KEY_SHARE0_3::Register>),
        (0x40 => key_share0_4: WriteOnly<u32, KEY_SHARE0_4::Register>),
        (0x44 => key_share0_5: WriteOnly<u32, KEY_SHARE0_5::Register>),
        (0x48 => key_share0_6: WriteOnly<u32, KEY_SHARE0_6::Register>),
        (0x4c => key_share0_7: WriteOnly<u32, KEY_SHARE0_7::Register>),
        (0x50 => key_share0_8: WriteOnly<u32, KEY_SHARE0_8::Register>),
        (0x54 => key_share0_9: WriteOnly<u32, KEY_SHARE0_9::Register>),
        (0x58 => key_share0_10: WriteOnly<u32, KEY_SHARE0_10::Register>),
        (0x5c => key_share0_11: WriteOnly<u32, KEY_SHARE0_11::Register>),
        (0x60 => key_share0_12: WriteOnly<u32, KEY_SHARE0_12::Register>),
        (0x64 => key_share0_13: WriteOnly<u32, KEY_SHARE0_13::Register>),
        (0x68 => key_share0_14: WriteOnly<u32, KEY_SHARE0_14::Register>),
        (0x6c => key_share0_15: WriteOnly<u32, KEY_SHARE0_15::Register>),
        (0x70 => key_share1_0: WriteOnly<u32, KEY_SHARE1_0::Register>),
        (0x74 => key_share1_1: WriteOnly<u32, KEY_SHARE1_1::Register>),
        (0x78 => key_share1_2: WriteOnly<u32, KEY_SHARE1_2::Register>),
        (0x7c => key_share1_3: WriteOnly<u32, KEY_SHARE1_3::Register>),
        (0x80 => key_share1_4: WriteOnly<u32, KEY_SHARE1_4::Register>),
        (0x84 => key_share1_5: WriteOnly<u32, KEY_SHARE1_5::Register>),
        (0x88 => key_share1_6: WriteOnly<u32, KEY_SHARE1_6::Register>),
        (0x8c => key_share1_7: WriteOnly<u32, KEY_SHARE1_7::Register>),
        (0x90 => key_share1_8: WriteOnly<u32, KEY_SHARE1_8::Register>),
        (0x94 => key_share1_9: WriteOnly<u32, KEY_SHARE1_9::Register>),
        (0x98 => key_share1_10: WriteOnly<u32, KEY_SHARE1_10::Register>),
        (0x9c => key_share1_11: WriteOnly<u32, KEY_SHARE1_11::Register>),
        (0xa0 => key_share1_12: WriteOnly<u32, KEY_SHARE1_12::Register>),
        (0xa4 => key_share1_13: WriteOnly<u32, KEY_SHARE1_13::Register>),
        (0xa8 => key_share1_14: WriteOnly<u32, KEY_SHARE1_14::Register>),
        (0xac => key_share1_15: WriteOnly<u32, KEY_SHARE1_15::Register>),
        (0xb0 => key_len: WriteOnly<u32, KEY_LEN::Register>),
        (0xb4 => prefix_0: ReadWrite<u32, PREFIX_0::Register>),
        (0xb8 => prefix_1: ReadWrite<u32, PREFIX_1::Register>),
        (0xbc => prefix_2: ReadWrite<u32, PREFIX_2::Register>),
        (0xc0 => prefix_3: ReadWrite<u32, PREFIX_3::Register>),
        (0xc4 => prefix_4: ReadWrite<u32, PREFIX_4::Register>),
        (0xc8 => prefix_5: ReadWrite<u32, PREFIX_5::Register>),
        (0xcc => prefix_6: ReadWrite<u32, PREFIX_6::Register>),
        (0xd0 => prefix_7: ReadWrite<u32, PREFIX_7::Register>),
        (0xd4 => prefix_8: ReadWrite<u32, PREFIX_8::Register>),
        (0xd8 => prefix_9: ReadWrite<u32, PREFIX_9::Register>),
        (0xdc => prefix_10: ReadWrite<u32, PREFIX_10::Register>),
        (0xe0 => err_code: ReadOnly<u32, ERR_CODE::Register>),
    }
}

register_bitfields![u32,
    INTR_STATE [
        KMAC_DONE OFFSET(0) NUMBITS(1) [],
        FIFO_EMPTY OFFSET(1) NUMBITS(1) [],
        KMAC_ERR OFFSET(2) NUMBITS(1) [],
    ],
    INTR_ENABLE [
        KMAC_DONE OFFSET(0) NUMBITS(1) [],
        FIFO_EMPTY OFFSET(1) NUMBITS(1) [],
        KMAC_ERR OFFSET(2) NUMBITS(1) [],
    ],
    INTR_TEST [
        KMAC_DONE OFFSET(0) NUMBITS(1) [],
        FIFO_EMPTY OFFSET(1) NUMBITS(1) [],
        KMAC_ERR OFFSET(2) NUMBITS(1) [],
    ],
    ALERT_TEST [
        FATAL_FAULT OFFSET(0) NUMBITS(1) [],
    ],
    CFG_REGWEN [
        EN OFFSET(0) NUMBITS(1) [],
    ],
    CFG [
        KMAC_EN OFFSET(0) NUMBITS(1) [],
        KSTRENGTH OFFSET(1) NUMBITS(3) [
            L128 = 0,
            L224 = 1,
            L256 = 2,
            L384 = 3,
            L512 = 4,
        ],
        MODE OFFSET(4) NUMBITS(2) [
            SHA3 = 0,
            SHAKE = 2,
            CSHAKE = 3,
        ],
        MSG_ENDIANNESS OFFSET(8) NUMBITS(1) [],
        STATE_ENDIANNESS OFFSET(9) NUMBITS(1) [],
        SIDELOAD OFFSET(12) NUMBITS(1) [],
        ENTROPY_MODE OFFSET(16) NUMBITS(2) [
            IDLE_MODE = 0,
            EDN_MODE = 1,
            SW_MODE = 2,
        ],
        ENTROPY_FAST_PROCESS OFFSET(19) NUMBITS(1) [],
        ENTROPY_READY OFFSET(24) NUMBITS(1) [],
        ERR_PROCESSED OFFSET(25) NUMBITS(1) [],
    ],
    CMD [
        CMD OFFSET(0) NUMBITS(4) [
            START = 1,
            PROCESS = 2,
            RUN = 4,
            DONE = 8,
        ],
        ENTROPY_REQ OFFSET(8) NUMBITS(1) [],
        HASH_CNT_CLR OFFSET(9) NUMBITS(1) [],
    ],
    STATUS [
        SHA3_IDLE OFFSET(0) NUMBITS(1) [],
        SHA3_ABSORB OFFSET(1) NUMBITS(1) [],
        SHA3_SQUEEZE OFFSET(2) NUMBITS(1) [],
        FIFO_DEPTH OFFSET(8) NUMBITS(5) [],
        FIFO_EMPTY OFFSET(14) NUMBITS(1) [],
        FIFO_FULL OFFSET(15) NUMBITS(1) [],
    ],
    ENTROPY_PERIOD [
        PRESCALER OFFSET(0) NUMBITS(10) [],
        WAIT_TIMER OFFSET(16) NUMBITS(16) [],
    ],
    ENTROPY_REFRESH [
        THRESHOLD OFFSET(0) NUMBITS(10) [],
        HASH_CNT OFFSET(16) NUMBITS(10) [],
    ],
    ENTROPY_SEED_LOWER [
        SEED OFFSET(0) NUMBITS(32) [],
    ],
    ENTROPY_SEED_UPPER [
        SEED OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE0_0 [
        KEY_0 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE0_1 [
        KEY_1 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE0_2 [
        KEY_2 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE0_3 [
        KEY_3 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE0_4 [
        KEY_4 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE0_5 [
        KEY_5 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE0_6 [
        KEY_6 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE0_7 [
        KEY_7 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE0_8 [
        KEY_8 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE0_9 [
        KEY_9 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE0_10 [
        KEY_10 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE0_11 [
        KEY_11 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE0_12 [
        KEY_12 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE0_13 [
        KEY_13 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE0_14 [
        KEY_14 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE0_15 [
        KEY_15 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE1_0 [
        KEY_0 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE1_1 [
        KEY_1 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE1_2 [
        KEY_2 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE1_3 [
        KEY_3 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE1_4 [
        KEY_4 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE1_5 [
        KEY_5 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE1_6 [
        KEY_6 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE1_7 [
        KEY_7 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE1_8 [
        KEY_8 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE1_9 [
        KEY_9 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE1_10 [
        KEY_10 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE1_11 [
        KEY_11 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE1_12 [
        KEY_12 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE1_13 [
        KEY_13 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE1_14 [
        KEY_14 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_SHARE1_15 [
        KEY_15 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_LEN [
        LEN OFFSET(0) NUMBITS(3) [
            KEY128 = 0,
            KEY192 = 1,
            KEY256 = 2,
            KEY384 = 3,
            KEY512 = 4,
        ],
    ],
    PREFIX_0 [
        PREFIX_0 OFFSET(0) NUMBITS(32) [],
    ],
    PREFIX_1 [
        PREFIX_1 OFFSET(0) NUMBITS(32) [],
    ],
    PREFIX_2 [
        PREFIX_2 OFFSET(0) NUMBITS(32) [],
    ],
    PREFIX_3 [
        PREFIX_3 OFFSET(0) NUMBITS(32) [],
    ],
    PREFIX_4 [
        PREFIX_4 OFFSET(0) NUMBITS(32) [],
    ],
    PREFIX_5 [
        PREFIX_5 OFFSET(0) NUMBITS(32) [],
    ],
    PREFIX_6 [
        PREFIX_6 OFFSET(0) NUMBITS(32) [],
    ],
    PREFIX_7 [
        PREFIX_7 OFFSET(0) NUMBITS(32) [],
    ],
    PREFIX_8 [
        PREFIX_8 OFFSET(0) NUMBITS(32) [],
    ],
    PREFIX_9 [
        PREFIX_9 OFFSET(0) NUMBITS(32) [],
    ],
    PREFIX_10 [
        PREFIX_10 OFFSET(0) NUMBITS(32) [],
    ],
    ERR_CODE [
        ERR_CODE OFFSET(0) NUMBITS(32) [],
    ],
];

// Number of words for the secret key
pub const KMAC_PARAM_NUM_WORDS_KEY: u32 = 16;

// Number of words for Encoded NsPrefix.
pub const KMAC_PARAM_NUM_WORDS_PREFIX: u32 = 11;

// Number of alerts
pub const KMAC_PARAM_NUM_ALERTS: u32 = 1;

// Register width
pub const KMAC_PARAM_REG_WIDTH: u32 = 32;

// KMAC Secret Key
pub const KMAC_KEY_SHARE0_KEY_FIELD_WIDTH: u32 = 32;
pub const KMAC_KEY_SHARE0_KEY_FIELDS_PER_REG: u32 = 1;
pub const KMAC_KEY_SHARE0_MULTIREG_COUNT: u32 = 16;

// KMAC Secret Key, 2nd share.
pub const KMAC_KEY_SHARE1_KEY_FIELD_WIDTH: u32 = 32;
pub const KMAC_KEY_SHARE1_KEY_FIELDS_PER_REG: u32 = 1;
pub const KMAC_KEY_SHARE1_MULTIREG_COUNT: u32 = 16;

// cSHAKE Prefix register.
pub const KMAC_PREFIX_PREFIX_FIELD_WIDTH: u32 = 32;
pub const KMAC_PREFIX_PREFIX_FIELDS_PER_REG: u32 = 1;
pub const KMAC_PREFIX_MULTIREG_COUNT: u32 = 11;

// Memory area: Keccak State (1600 bit) memory.
pub const KMAC_STATE_REG_OFFSET: usize = 0x400;
pub const KMAC_STATE_SIZE_WORDS: u32 = 128;
pub const KMAC_STATE_SIZE_BYTES: u32 = 512;
// Memory area: Message FIFO.
pub const KMAC_MSG_FIFO_REG_OFFSET: usize = 0x800;
pub const KMAC_MSG_FIFO_SIZE_WORDS: u32 = 512;
pub const KMAC_MSG_FIFO_SIZE_BYTES: u32 = 2048;
// End generated register constants for kmac

