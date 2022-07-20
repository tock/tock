// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright lowRISC contributors 2023.

// Generated register constants for kmac.
// Built for Earlgrey-M2.5.1-RC1-438-gacc67de99
// https://github.com/lowRISC/opentitan/tree/acc67de992ee8de5f2481b1b9580679850d8b5f5
// Tree status: clean
// Build date: 2023-08-08T00:15:38

// Original reference file: hw/ip/kmac/data/kmac.hjson
use kernel::utilities::registers::ReadOnly;
use kernel::utilities::registers::ReadWrite;
use kernel::utilities::registers::WriteOnly;
use kernel::utilities::registers::{register_bitfields, register_structs};
/// Number of words for the secret key
pub const KMAC_PARAM_NUM_WORDS_KEY: u32 = 16;
/// Number of words for Encoded NsPrefix.
pub const KMAC_PARAM_NUM_WORDS_PREFIX: u32 = 11;
/// Number of entries in the message FIFO. Must match kmac_pkg::MsgFifoDepth.
pub const KMAC_PARAM_NUM_ENTRIES_MSG_FIFO: u32 = 10;
/// Number of bytes in a single entry of the message FIFO. Must match kmac_pkg::MsgWidth.
pub const KMAC_PARAM_NUM_BYTES_MSG_FIFO_ENTRY: u32 = 8;
/// Number of words for the LFSR seed used for entropy generation
pub const KMAC_PARAM_NUM_SEEDS_ENTROPY_LFSR: u32 = 5;
/// Number of alerts
pub const KMAC_PARAM_NUM_ALERTS: u32 = 2;
/// Register width
pub const KMAC_PARAM_REG_WIDTH: u32 = 32;

register_structs! {
    pub KmacRegisters {
        /// Interrupt State Register
        (0x0000 => pub(crate) intr_state: ReadWrite<u32, INTR::Register>),
        /// Interrupt Enable Register
        (0x0004 => pub(crate) intr_enable: ReadWrite<u32, INTR::Register>),
        /// Interrupt Test Register
        (0x0008 => pub(crate) intr_test: ReadWrite<u32, INTR::Register>),
        /// Alert Test Register
        (0x000c => pub(crate) alert_test: ReadWrite<u32, ALERT_TEST::Register>),
        /// Controls the configurability of !!CFG_SHADOWED register.
        (0x0010 => pub(crate) cfg_regwen: ReadWrite<u32, CFG_REGWEN::Register>),
        /// KMAC Configuration register.
        (0x0014 => pub(crate) cfg_shadowed: ReadWrite<u32, CFG_SHADOWED::Register>),
        /// KMAC/ SHA3 command register.
        (0x0018 => pub(crate) cmd: ReadWrite<u32, CMD::Register>),
        /// KMAC/SHA3 Status register.
        (0x001c => pub(crate) status: ReadWrite<u32, STATUS::Register>),
        /// Entropy Timer Periods.
        (0x0020 => pub(crate) entropy_period: ReadWrite<u32, ENTROPY_PERIOD::Register>),
        /// Entropy Refresh Counter
        (0x0024 => pub(crate) entropy_refresh_hash_cnt: ReadWrite<u32, ENTROPY_REFRESH_HASH_CNT::Register>),
        /// Entropy Refresh Threshold
        (0x0028 => pub(crate) entropy_refresh_threshold_shadowed: ReadWrite<u32, ENTROPY_REFRESH_THRESHOLD_SHADOWED::Register>),
        /// Entropy Seed
        (0x002c => pub(crate) entropy_seed: [ReadWrite<u32, ENTROPY_SEED::Register>; 5]),
        /// KMAC Secret Key
        (0x0040 => pub(crate) key_share0: [ReadWrite<u32, KEY_SHARE0::Register>; 16]),
        /// KMAC Secret Key, 2nd share.
        (0x0080 => pub(crate) key_share1: [ReadWrite<u32, KEY_SHARE1::Register>; 16]),
        /// Secret Key length in bit.
        (0x00c0 => pub(crate) key_len: ReadWrite<u32, KEY_LEN::Register>),
        /// cSHAKE Prefix register.
        (0x00c4 => pub(crate) prefix: [ReadWrite<u32, PREFIX::Register>; 11]),
        /// KMAC/SHA3 Error Code
        (0x00f0 => pub(crate) err_code: ReadWrite<u32, ERR_CODE::Register>),
        (0x00f4 => _reserved1),
        /// Memory area: Keccak State (1600 bit) memory.
        (0x0400 => pub(crate) state: [ReadOnly<u32>; 128]),
        (0x0600 => _reserved2),
        /// Memory area: Message FIFO.
        (0x0800 => pub(crate) msg_fifo: [WriteOnly<u32>; 512]),
        (0x1000 => @END),
    }
}

register_bitfields![u32,
    /// Common Interrupt Offsets
    pub(crate) INTR [
        KMAC_DONE OFFSET(0) NUMBITS(1) [],
        FIFO_EMPTY OFFSET(1) NUMBITS(1) [],
        KMAC_ERR OFFSET(2) NUMBITS(1) [],
    ],
    pub(crate) ALERT_TEST [
        RECOV_OPERATION_ERR OFFSET(0) NUMBITS(1) [],
        FATAL_FAULT_ERR OFFSET(1) NUMBITS(1) [],
    ],
    pub(crate) CFG_REGWEN [
        EN OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) CFG_SHADOWED [
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
        MSG_MASK OFFSET(20) NUMBITS(1) [],
        ENTROPY_READY OFFSET(24) NUMBITS(1) [],
        ERR_PROCESSED OFFSET(25) NUMBITS(1) [],
        EN_UNSUPPORTED_MODESTRENGTH OFFSET(26) NUMBITS(1) [],
    ],
    pub(crate) CMD [
        CMD OFFSET(0) NUMBITS(6) [
            START = 29,
            PROCESS = 46,
            RUN = 49,
            DONE = 22,
        ],
        ENTROPY_REQ OFFSET(8) NUMBITS(1) [],
        HASH_CNT_CLR OFFSET(9) NUMBITS(1) [],
    ],
    pub(crate) STATUS [
        SHA3_IDLE OFFSET(0) NUMBITS(1) [],
        SHA3_ABSORB OFFSET(1) NUMBITS(1) [],
        SHA3_SQUEEZE OFFSET(2) NUMBITS(1) [],
        FIFO_DEPTH OFFSET(8) NUMBITS(5) [],
        FIFO_EMPTY OFFSET(14) NUMBITS(1) [],
        FIFO_FULL OFFSET(15) NUMBITS(1) [],
        ALERT_FATAL_FAULT OFFSET(16) NUMBITS(1) [],
        ALERT_RECOV_CTRL_UPDATE_ERR OFFSET(17) NUMBITS(1) [],
    ],
    pub(crate) ENTROPY_PERIOD [
        PRESCALER OFFSET(0) NUMBITS(10) [],
        WAIT_TIMER OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) ENTROPY_REFRESH_HASH_CNT [
        HASH_CNT OFFSET(0) NUMBITS(10) [],
    ],
    pub(crate) ENTROPY_REFRESH_THRESHOLD_SHADOWED [
        THRESHOLD OFFSET(0) NUMBITS(10) [],
    ],
    pub(crate) ENTROPY_SEED [
        SEED_0 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) KEY_SHARE0 [
        KEY_0 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) KEY_SHARE1 [
        KEY_0 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) KEY_LEN [
        LEN OFFSET(0) NUMBITS(3) [
            KEY128 = 0,
            KEY192 = 1,
            KEY256 = 2,
            KEY384 = 3,
            KEY512 = 4,
        ],
    ],
    pub(crate) PREFIX [
        PREFIX_0 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) ERR_CODE [
        ERR_CODE OFFSET(0) NUMBITS(32) [],
    ],
];

// End generated register constants for kmac
