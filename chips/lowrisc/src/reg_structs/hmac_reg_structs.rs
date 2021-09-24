// Generated register struct for hmac

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
    pub HmacRegisters {
        (0x0 => intr_state: ReadWrite<u32, INTR_STATE::Register>),
        (0x4 => intr_enable: ReadWrite<u32, INTR_ENABLE::Register>),
        (0x8 => intr_test: WriteOnly<u32, INTR_TEST::Register>),
        (0xc => alert_test: WriteOnly<u32, ALERT_TEST::Register>),
        (0x10 => cfg: ReadWrite<u32, CFG::Register>),
        (0x14 => cmd: WriteOnly<u32, CMD::Register>),
        (0x18 => status: ReadOnly<u32, STATUS::Register>),
        (0x1c => err_code: ReadOnly<u32, ERR_CODE::Register>),
        (0x20 => wipe_secret: WriteOnly<u32, WIPE_SECRET::Register>),
        (0x24 => key_0: WriteOnly<u32, KEY_0::Register>),
        (0x28 => key_1: WriteOnly<u32, KEY_1::Register>),
        (0x2c => key_2: WriteOnly<u32, KEY_2::Register>),
        (0x30 => key_3: WriteOnly<u32, KEY_3::Register>),
        (0x34 => key_4: WriteOnly<u32, KEY_4::Register>),
        (0x38 => key_5: WriteOnly<u32, KEY_5::Register>),
        (0x3c => key_6: WriteOnly<u32, KEY_6::Register>),
        (0x40 => key_7: WriteOnly<u32, KEY_7::Register>),
        (0x44 => digest_0: ReadOnly<u32, DIGEST_0::Register>),
        (0x48 => digest_1: ReadOnly<u32, DIGEST_1::Register>),
        (0x4c => digest_2: ReadOnly<u32, DIGEST_2::Register>),
        (0x50 => digest_3: ReadOnly<u32, DIGEST_3::Register>),
        (0x54 => digest_4: ReadOnly<u32, DIGEST_4::Register>),
        (0x58 => digest_5: ReadOnly<u32, DIGEST_5::Register>),
        (0x5c => digest_6: ReadOnly<u32, DIGEST_6::Register>),
        (0x60 => digest_7: ReadOnly<u32, DIGEST_7::Register>),
        (0x64 => msg_length_lower: ReadOnly<u32, MSG_LENGTH_LOWER::Register>),
        (0x68 => msg_length_upper: ReadOnly<u32, MSG_LENGTH_UPPER::Register>),
    }
}

register_bitfields![u32,
    INTR_STATE [
        HMAC_DONE OFFSET(0) NUMBITS(1) [],
        FIFO_EMPTY OFFSET(1) NUMBITS(1) [],
        HMAC_ERR OFFSET(2) NUMBITS(1) [],
    ],
    INTR_ENABLE [
        HMAC_DONE OFFSET(0) NUMBITS(1) [],
        FIFO_EMPTY OFFSET(1) NUMBITS(1) [],
        HMAC_ERR OFFSET(2) NUMBITS(1) [],
    ],
    INTR_TEST [
        HMAC_DONE OFFSET(0) NUMBITS(1) [],
        FIFO_EMPTY OFFSET(1) NUMBITS(1) [],
        HMAC_ERR OFFSET(2) NUMBITS(1) [],
    ],
    ALERT_TEST [
        FATAL_FAULT OFFSET(0) NUMBITS(1) [],
    ],
    CFG [
        HMAC_EN OFFSET(0) NUMBITS(1) [],
        SHA_EN OFFSET(1) NUMBITS(1) [],
        ENDIAN_SWAP OFFSET(2) NUMBITS(1) [],
        DIGEST_SWAP OFFSET(3) NUMBITS(1) [],
    ],
    CMD [
        HASH_START OFFSET(0) NUMBITS(1) [],
        HASH_PROCESS OFFSET(1) NUMBITS(1) [],
    ],
    STATUS [
        FIFO_EMPTY OFFSET(0) NUMBITS(1) [],
        FIFO_FULL OFFSET(1) NUMBITS(1) [],
        FIFO_DEPTH OFFSET(4) NUMBITS(5) [],
    ],
    ERR_CODE [
        ERR_CODE OFFSET(0) NUMBITS(32) [],
    ],
    WIPE_SECRET [
        SECRET OFFSET(0) NUMBITS(32) [],
    ],
    KEY_0 [
        KEY_0 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_1 [
        KEY_1 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_2 [
        KEY_2 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_3 [
        KEY_3 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_4 [
        KEY_4 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_5 [
        KEY_5 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_6 [
        KEY_6 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_7 [
        KEY_7 OFFSET(0) NUMBITS(32) [],
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
    MSG_LENGTH_LOWER [
        V OFFSET(0) NUMBITS(32) [],
    ],
    MSG_LENGTH_UPPER [
        V OFFSET(0) NUMBITS(32) [],
    ],
];

// Number of words for digest/ key
pub const HMAC_PARAM_NUM_WORDS: u32 = 8;

// Number of alerts
pub const HMAC_PARAM_NUM_ALERTS: u32 = 1;

// Register width
pub const HMAC_PARAM_REG_WIDTH: u32 = 32;

// HMAC Secret Key
pub const HMAC_KEY_KEY_FIELD_WIDTH: u32 = 32;
pub const HMAC_KEY_KEY_FIELDS_PER_REG: u32 = 1;
pub const HMAC_KEY_MULTIREG_COUNT: u32 = 8;

// Digest output. If HMAC is disabled, the register shows result of SHA256
pub const HMAC_DIGEST_DIGEST_FIELD_WIDTH: u32 = 32;
pub const HMAC_DIGEST_DIGEST_FIELDS_PER_REG: u32 = 1;
pub const HMAC_DIGEST_MULTIREG_COUNT: u32 = 8;

// Memory area: Message FIFO. Any write to this window will be appended to
// the FIFO. Only the lower [1:0] bits of the address matter to writes within
// the window (for correctly dealing with non 32-bit writes)
pub const HMAC_MSG_FIFO_REG_OFFSET: usize = 0x800;
pub const HMAC_MSG_FIFO_SIZE_WORDS: u32 = 512;
pub const HMAC_MSG_FIFO_SIZE_BYTES: u32 = 2048;
// End generated register constants for hmac

