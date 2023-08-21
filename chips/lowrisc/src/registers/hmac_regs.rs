// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright lowRISC contributors 2023.

// Generated register constants for hmac.
// Built for Earlgrey-M2.5.1-RC1-438-gacc67de99
// https://github.com/lowRISC/opentitan/tree/acc67de992ee8de5f2481b1b9580679850d8b5f5
// Tree status: clean
// Build date: 2023-08-08T00:15:38

// Original reference file: hw/ip/hmac/data/hmac.hjson
use kernel::utilities::registers::ReadWrite;
use kernel::utilities::registers::WriteOnly;
use kernel::utilities::registers::{register_bitfields, register_structs};
/// Number of words for digest/ key
pub const HMAC_PARAM_NUM_WORDS: u32 = 8;
/// Number of alerts
pub const HMAC_PARAM_NUM_ALERTS: u32 = 1;
/// Register width
pub const HMAC_PARAM_REG_WIDTH: u32 = 32;

register_structs! {
    pub HmacRegisters {
        /// Interrupt State Register
        (0x0000 => pub(crate) intr_state: ReadWrite<u32, INTR::Register>),
        /// Interrupt Enable Register
        (0x0004 => pub(crate) intr_enable: ReadWrite<u32, INTR::Register>),
        /// Interrupt Test Register
        (0x0008 => pub(crate) intr_test: ReadWrite<u32, INTR::Register>),
        /// Alert Test Register
        (0x000c => pub(crate) alert_test: ReadWrite<u32, ALERT_TEST::Register>),
        /// HMAC Configuration register.
        (0x0010 => pub(crate) cfg: ReadWrite<u32, CFG::Register>),
        /// HMAC command register
        (0x0014 => pub(crate) cmd: ReadWrite<u32, CMD::Register>),
        /// HMAC Status register
        (0x0018 => pub(crate) status: ReadWrite<u32, STATUS::Register>),
        /// HMAC Error Code
        (0x001c => pub(crate) err_code: ReadWrite<u32, ERR_CODE::Register>),
        /// Randomize internal secret registers.
        (0x0020 => pub(crate) wipe_secret: ReadWrite<u32, WIPE_SECRET::Register>),
        /// HMAC Secret Key
        (0x0024 => pub(crate) key: [ReadWrite<u32, KEY::Register>; 8]),
        /// Digest output. If HMAC is disabled, the register shows result of SHA256
        (0x0044 => pub(crate) digest: [ReadWrite<u32, DIGEST::Register>; 8]),
        /// Received Message Length calculated by the HMAC in bits [31:0]
        (0x0064 => pub(crate) msg_length_lower: ReadWrite<u32, MSG_LENGTH_LOWER::Register>),
        /// Received Message Length calculated by the HMAC in bits [63:32]
        (0x0068 => pub(crate) msg_length_upper: ReadWrite<u32, MSG_LENGTH_UPPER::Register>),
        (0x006c => _reserved1),
        /// Memory area: Message FIFO. Any write to this window will be appended to the FIFO. Only the
        /// lower [1:0] bits of the address matter to writes within the window (for correctly dealing
        /// with non 32-bit writes)
        (0x0800 => pub(crate) msg_fifo: [WriteOnly<u32>; 512]),
        (0x1000 => @END),
    }
}

register_bitfields![u32,
    /// Common Interrupt Offsets
    pub(crate) INTR [
        HMAC_DONE OFFSET(0) NUMBITS(1) [],
        FIFO_EMPTY OFFSET(1) NUMBITS(1) [],
        HMAC_ERR OFFSET(2) NUMBITS(1) [],
    ],
    pub(crate) ALERT_TEST [
        FATAL_FAULT OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) CFG [
        HMAC_EN OFFSET(0) NUMBITS(1) [],
        SHA_EN OFFSET(1) NUMBITS(1) [],
        ENDIAN_SWAP OFFSET(2) NUMBITS(1) [],
        DIGEST_SWAP OFFSET(3) NUMBITS(1) [],
    ],
    pub(crate) CMD [
        HASH_START OFFSET(0) NUMBITS(1) [],
        HASH_PROCESS OFFSET(1) NUMBITS(1) [],
    ],
    pub(crate) STATUS [
        FIFO_EMPTY OFFSET(0) NUMBITS(1) [],
        FIFO_FULL OFFSET(1) NUMBITS(1) [],
        FIFO_DEPTH OFFSET(4) NUMBITS(5) [],
    ],
    pub(crate) ERR_CODE [
        ERR_CODE OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) WIPE_SECRET [
        SECRET OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) KEY [
        KEY_0 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) DIGEST [
        DIGEST_0 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) MSG_LENGTH_LOWER [
        V OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) MSG_LENGTH_UPPER [
        V OFFSET(0) NUMBITS(32) [],
    ],
];

// End generated register constants for hmac
