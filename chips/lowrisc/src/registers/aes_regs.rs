// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright lowRISC contributors 2023.

// Generated register constants for aes.
// Built for Earlgrey-M2.5.1-RC1-438-gacc67de99
// https://github.com/lowRISC/opentitan/tree/acc67de992ee8de5f2481b1b9580679850d8b5f5
// Tree status: clean
// Build date: 2023-08-08T00:15:38

// Original reference file: hw/ip/aes/data/aes.hjson
use kernel::utilities::registers::ReadWrite;
use kernel::utilities::registers::{register_bitfields, register_structs};
/// Number registers for key
pub const AES_PARAM_NUM_REGS_KEY: u32 = 8;
/// Number registers for initialization vector
pub const AES_PARAM_NUM_REGS_IV: u32 = 4;
/// Number registers for input and output data
pub const AES_PARAM_NUM_REGS_DATA: u32 = 4;
/// Number of alerts
pub const AES_PARAM_NUM_ALERTS: u32 = 2;
/// Register width
pub const AES_PARAM_REG_WIDTH: u32 = 32;

register_structs! {
    pub AesRegisters {
        /// Alert Test Register
        (0x0000 => pub(crate) alert_test: ReadWrite<u32, ALERT_TEST::Register>),
        /// Initial Key Registers Share 0.
        (0x0004 => pub(crate) key_share0: [ReadWrite<u32, KEY_SHARE0::Register>; 8]),
        /// Initial Key Registers Share 1.
        (0x0024 => pub(crate) key_share1: [ReadWrite<u32, KEY_SHARE1::Register>; 8]),
        /// Initialization Vector Registers.
        (0x0044 => pub(crate) iv: [ReadWrite<u32, IV::Register>; 4]),
        /// Input Data Registers.
        (0x0054 => pub(crate) data_in: [ReadWrite<u32, DATA_IN::Register>; 4]),
        /// Output Data Register.
        (0x0064 => pub(crate) data_out: [ReadWrite<u32, DATA_OUT::Register>; 4]),
        /// Control Register.
        (0x0074 => pub(crate) ctrl_shadowed: ReadWrite<u32, CTRL_SHADOWED::Register>),
        /// Auxiliary Control Register.
        (0x0078 => pub(crate) ctrl_aux_shadowed: ReadWrite<u32, CTRL_AUX_SHADOWED::Register>),
        /// Lock bit for Auxiliary Control Register.
        (0x007c => pub(crate) ctrl_aux_regwen: ReadWrite<u32, CTRL_AUX_REGWEN::Register>),
        /// Trigger Register.
        (0x0080 => pub(crate) trigger: ReadWrite<u32, TRIGGER::Register>),
        /// Status Register
        (0x0084 => pub(crate) status: ReadWrite<u32, STATUS::Register>),
        (0x0088 => @END),
    }
}

register_bitfields![u32,
    pub(crate) ALERT_TEST [
        RECOV_CTRL_UPDATE_ERR OFFSET(0) NUMBITS(1) [],
        FATAL_FAULT OFFSET(1) NUMBITS(1) [],
    ],
    pub(crate) KEY_SHARE0 [
        KEY_SHARE0_0 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) KEY_SHARE1 [
        KEY_SHARE1_0 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) IV [
        IV_0 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) DATA_IN [
        DATA_IN_0 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) DATA_OUT [
        DATA_OUT_0 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) CTRL_SHADOWED [
        OPERATION OFFSET(0) NUMBITS(2) [
            AES_ENC = 1,
            AES_DEC = 2,
        ],
        MODE OFFSET(2) NUMBITS(6) [
            AES_ECB = 1,
            AES_CBC = 2,
            AES_CFB = 4,
            AES_OFB = 8,
            AES_CTR = 16,
            AES_NONE = 32,
        ],
        KEY_LEN OFFSET(8) NUMBITS(3) [
            AES_128 = 1,
            AES_192 = 2,
            AES_256 = 4,
        ],
        SIDELOAD OFFSET(11) NUMBITS(1) [],
        PRNG_RESEED_RATE OFFSET(12) NUMBITS(3) [
            PER_1 = 1,
            PER_64 = 2,
            PER_8K = 4,
        ],
        MANUAL_OPERATION OFFSET(15) NUMBITS(1) [],
    ],
    pub(crate) CTRL_AUX_SHADOWED [
        KEY_TOUCH_FORCES_RESEED OFFSET(0) NUMBITS(1) [],
        FORCE_MASKS OFFSET(1) NUMBITS(1) [],
    ],
    pub(crate) CTRL_AUX_REGWEN [
        CTRL_AUX_REGWEN OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) TRIGGER [
        START OFFSET(0) NUMBITS(1) [],
        KEY_IV_DATA_IN_CLEAR OFFSET(1) NUMBITS(1) [],
        DATA_OUT_CLEAR OFFSET(2) NUMBITS(1) [],
        PRNG_RESEED OFFSET(3) NUMBITS(1) [],
    ],
    pub(crate) STATUS [
        IDLE OFFSET(0) NUMBITS(1) [],
        STALL OFFSET(1) NUMBITS(1) [],
        OUTPUT_LOST OFFSET(2) NUMBITS(1) [],
        OUTPUT_VALID OFFSET(3) NUMBITS(1) [],
        INPUT_READY OFFSET(4) NUMBITS(1) [],
        ALERT_RECOV_CTRL_UPDATE_ERR OFFSET(5) NUMBITS(1) [],
        ALERT_FATAL_FAULT OFFSET(6) NUMBITS(1) [],
    ],
];

// End generated register constants for aes
