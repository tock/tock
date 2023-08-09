// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright lowRISC contributors 2023.

// Generated register constants for csrng.
// Built for Earlgrey-M2.5.1-RC1-438-gacc67de99
// https://github.com/lowRISC/opentitan/tree/acc67de992ee8de5f2481b1b9580679850d8b5f5
// Tree status: clean
// Build date: 2023-08-08T00:15:38

// Original reference file: hw/ip/csrng/data/csrng.hjson
use kernel::utilities::registers::ReadWrite;
use kernel::utilities::registers::{register_bitfields, register_structs};
/// Number of alerts
pub const CSRNG_PARAM_NUM_ALERTS: u32 = 2;
/// Register width
pub const CSRNG_PARAM_REG_WIDTH: u32 = 32;

register_structs! {
    pub CsrngRegisters {
        /// Interrupt State Register
        (0x0000 => pub(crate) intr_state: ReadWrite<u32, INTR::Register>),
        /// Interrupt Enable Register
        (0x0004 => pub(crate) intr_enable: ReadWrite<u32, INTR::Register>),
        /// Interrupt Test Register
        (0x0008 => pub(crate) intr_test: ReadWrite<u32, INTR::Register>),
        /// Alert Test Register
        (0x000c => pub(crate) alert_test: ReadWrite<u32, ALERT_TEST::Register>),
        /// Register write enable for all control registers
        (0x0010 => pub(crate) regwen: ReadWrite<u32, REGWEN::Register>),
        /// Control register
        (0x0014 => pub(crate) ctrl: ReadWrite<u32, CTRL::Register>),
        /// Command request register
        (0x0018 => pub(crate) cmd_req: ReadWrite<u32, CMD_REQ::Register>),
        /// Application interface command status register
        (0x001c => pub(crate) sw_cmd_sts: ReadWrite<u32, SW_CMD_STS::Register>),
        /// Generate bits returned valid register
        (0x0020 => pub(crate) genbits_vld: ReadWrite<u32, GENBITS_VLD::Register>),
        /// Generate bits returned register
        (0x0024 => pub(crate) genbits: ReadWrite<u32, GENBITS::Register>),
        /// Internal state number register
        (0x0028 => pub(crate) int_state_num: ReadWrite<u32, INT_STATE_NUM::Register>),
        /// Internal state read access register
        (0x002c => pub(crate) int_state_val: ReadWrite<u32, INT_STATE_VAL::Register>),
        /// Hardware instance exception status register
        (0x0030 => pub(crate) hw_exc_sts: ReadWrite<u32, HW_EXC_STS::Register>),
        /// Recoverable alert status register
        (0x0034 => pub(crate) recov_alert_sts: ReadWrite<u32, RECOV_ALERT_STS::Register>),
        /// Hardware detection of error conditions status register
        (0x0038 => pub(crate) err_code: ReadWrite<u32, ERR_CODE::Register>),
        /// Test error conditions register
        (0x003c => pub(crate) err_code_test: ReadWrite<u32, ERR_CODE_TEST::Register>),
        /// Main state machine state debug register
        (0x0040 => pub(crate) main_sm_state: ReadWrite<u32, MAIN_SM_STATE::Register>),
        (0x0044 => @END),
    }
}

register_bitfields![u32,
    /// Common Interrupt Offsets
    pub(crate) INTR [
        CS_CMD_REQ_DONE OFFSET(0) NUMBITS(1) [],
        CS_ENTROPY_REQ OFFSET(1) NUMBITS(1) [],
        CS_HW_INST_EXC OFFSET(2) NUMBITS(1) [],
        CS_FATAL_ERR OFFSET(3) NUMBITS(1) [],
    ],
    pub(crate) ALERT_TEST [
        RECOV_ALERT OFFSET(0) NUMBITS(1) [],
        FATAL_ALERT OFFSET(1) NUMBITS(1) [],
    ],
    pub(crate) REGWEN [
        REGWEN OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) CTRL [
        ENABLE OFFSET(0) NUMBITS(4) [],
        SW_APP_ENABLE OFFSET(4) NUMBITS(4) [],
        READ_INT_STATE OFFSET(8) NUMBITS(4) [],
    ],
    pub(crate) CMD_REQ [
        CMD_REQ OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) SW_CMD_STS [
        CMD_RDY OFFSET(0) NUMBITS(1) [],
        CMD_STS OFFSET(1) NUMBITS(1) [],
    ],
    pub(crate) GENBITS_VLD [
        GENBITS_VLD OFFSET(0) NUMBITS(1) [],
        GENBITS_FIPS OFFSET(1) NUMBITS(1) [],
    ],
    pub(crate) GENBITS [
        GENBITS OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) INT_STATE_NUM [
        INT_STATE_NUM OFFSET(0) NUMBITS(4) [],
    ],
    pub(crate) INT_STATE_VAL [
        INT_STATE_VAL OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) HW_EXC_STS [
        HW_EXC_STS OFFSET(0) NUMBITS(16) [],
    ],
    pub(crate) RECOV_ALERT_STS [
        ENABLE_FIELD_ALERT OFFSET(0) NUMBITS(1) [],
        SW_APP_ENABLE_FIELD_ALERT OFFSET(1) NUMBITS(1) [],
        READ_INT_STATE_FIELD_ALERT OFFSET(2) NUMBITS(1) [],
        ACMD_FLAG0_FIELD_ALERT OFFSET(3) NUMBITS(1) [],
        CS_BUS_CMP_ALERT OFFSET(12) NUMBITS(1) [],
        CS_MAIN_SM_ALERT OFFSET(13) NUMBITS(1) [],
    ],
    pub(crate) ERR_CODE [
        SFIFO_CMD_ERR OFFSET(0) NUMBITS(1) [],
        SFIFO_GENBITS_ERR OFFSET(1) NUMBITS(1) [],
        SFIFO_CMDREQ_ERR OFFSET(2) NUMBITS(1) [],
        SFIFO_RCSTAGE_ERR OFFSET(3) NUMBITS(1) [],
        SFIFO_KEYVRC_ERR OFFSET(4) NUMBITS(1) [],
        SFIFO_UPDREQ_ERR OFFSET(5) NUMBITS(1) [],
        SFIFO_BENCREQ_ERR OFFSET(6) NUMBITS(1) [],
        SFIFO_BENCACK_ERR OFFSET(7) NUMBITS(1) [],
        SFIFO_PDATA_ERR OFFSET(8) NUMBITS(1) [],
        SFIFO_FINAL_ERR OFFSET(9) NUMBITS(1) [],
        SFIFO_GBENCACK_ERR OFFSET(10) NUMBITS(1) [],
        SFIFO_GRCSTAGE_ERR OFFSET(11) NUMBITS(1) [],
        SFIFO_GGENREQ_ERR OFFSET(12) NUMBITS(1) [],
        SFIFO_GADSTAGE_ERR OFFSET(13) NUMBITS(1) [],
        SFIFO_GGENBITS_ERR OFFSET(14) NUMBITS(1) [],
        SFIFO_BLKENC_ERR OFFSET(15) NUMBITS(1) [],
        CMD_STAGE_SM_ERR OFFSET(20) NUMBITS(1) [],
        MAIN_SM_ERR OFFSET(21) NUMBITS(1) [],
        DRBG_GEN_SM_ERR OFFSET(22) NUMBITS(1) [],
        DRBG_UPDBE_SM_ERR OFFSET(23) NUMBITS(1) [],
        DRBG_UPDOB_SM_ERR OFFSET(24) NUMBITS(1) [],
        AES_CIPHER_SM_ERR OFFSET(25) NUMBITS(1) [],
        CMD_GEN_CNT_ERR OFFSET(26) NUMBITS(1) [],
        FIFO_WRITE_ERR OFFSET(28) NUMBITS(1) [],
        FIFO_READ_ERR OFFSET(29) NUMBITS(1) [],
        FIFO_STATE_ERR OFFSET(30) NUMBITS(1) [],
    ],
    pub(crate) ERR_CODE_TEST [
        ERR_CODE_TEST OFFSET(0) NUMBITS(5) [],
    ],
    pub(crate) MAIN_SM_STATE [
        MAIN_SM_STATE OFFSET(0) NUMBITS(8) [],
    ],
];

// End generated register constants for csrng
