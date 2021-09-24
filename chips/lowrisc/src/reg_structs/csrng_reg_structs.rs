// Generated register struct for csrng

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
    pub CsrngRegisters {
        (0x0 => intr_state: ReadWrite<u32, INTR_STATE::Register>),
        (0x4 => intr_enable: ReadWrite<u32, INTR_ENABLE::Register>),
        (0x8 => intr_test: WriteOnly<u32, INTR_TEST::Register>),
        (0xc => alert_test: WriteOnly<u32, ALERT_TEST::Register>),
        (0x10 => regwen: ReadWrite<u32, REGWEN::Register>),
        (0x14 => ctrl: ReadWrite<u32, CTRL::Register>),
        (0x18 => cmd_req: WriteOnly<u32, CMD_REQ::Register>),
        (0x1c => sw_cmd_sts: ReadOnly<u32, SW_CMD_STS::Register>),
        (0x20 => genbits_vld: ReadOnly<u32, GENBITS_VLD::Register>),
        (0x24 => genbits: ReadOnly<u32, GENBITS::Register>),
        (0x28 => int_state_num: ReadWrite<u32, INT_STATE_NUM::Register>),
        (0x2c => int_state_val: ReadOnly<u32, INT_STATE_VAL::Register>),
        (0x30 => hw_exc_sts: ReadWrite<u32, HW_EXC_STS::Register>),
        (0x34 => recov_alert_sts: ReadWrite<u32, RECOV_ALERT_STS::Register>),
        (0x38 => err_code: ReadOnly<u32, ERR_CODE::Register>),
        (0x3c => err_code_test: ReadWrite<u32, ERR_CODE_TEST::Register>),
        (0x40 => sel_tracking_sm: WriteOnly<u32, SEL_TRACKING_SM::Register>),
        (0x44 => tracking_sm_obs: ReadOnly<u32, TRACKING_SM_OBS::Register>),
    }
}

register_bitfields![u32,
    INTR_STATE [
        CS_CMD_REQ_DONE OFFSET(0) NUMBITS(1) [],
        CS_ENTROPY_REQ OFFSET(1) NUMBITS(1) [],
        CS_HW_INST_EXC OFFSET(2) NUMBITS(1) [],
        CS_FATAL_ERR OFFSET(3) NUMBITS(1) [],
    ],
    INTR_ENABLE [
        CS_CMD_REQ_DONE OFFSET(0) NUMBITS(1) [],
        CS_ENTROPY_REQ OFFSET(1) NUMBITS(1) [],
        CS_HW_INST_EXC OFFSET(2) NUMBITS(1) [],
        CS_FATAL_ERR OFFSET(3) NUMBITS(1) [],
    ],
    INTR_TEST [
        CS_CMD_REQ_DONE OFFSET(0) NUMBITS(1) [],
        CS_ENTROPY_REQ OFFSET(1) NUMBITS(1) [],
        CS_HW_INST_EXC OFFSET(2) NUMBITS(1) [],
        CS_FATAL_ERR OFFSET(3) NUMBITS(1) [],
    ],
    ALERT_TEST [
        RECOV_ALERT OFFSET(0) NUMBITS(1) [],
        FATAL_ALERT OFFSET(1) NUMBITS(1) [],
    ],
    REGWEN [
        REGWEN OFFSET(0) NUMBITS(1) [],
    ],
    CTRL [
        ENABLE OFFSET(0) NUMBITS(4) [],
        SW_APP_ENABLE OFFSET(4) NUMBITS(4) [],
        READ_INT_STATE OFFSET(8) NUMBITS(4) [],
    ],
    CMD_REQ [
        CMD_REQ OFFSET(0) NUMBITS(32) [],
    ],
    SW_CMD_STS [
        CMD_RDY OFFSET(0) NUMBITS(1) [],
        CMD_STS OFFSET(1) NUMBITS(1) [],
    ],
    GENBITS_VLD [
        GENBITS_VLD OFFSET(0) NUMBITS(1) [],
        GENBITS_FIPS OFFSET(1) NUMBITS(1) [],
    ],
    GENBITS [
        GENBITS OFFSET(0) NUMBITS(32) [],
    ],
    INT_STATE_NUM [
        INT_STATE_NUM OFFSET(0) NUMBITS(4) [],
    ],
    INT_STATE_VAL [
        INT_STATE_VAL OFFSET(0) NUMBITS(32) [],
    ],
    HW_EXC_STS [
        HW_EXC_STS OFFSET(0) NUMBITS(15) [],
    ],
    RECOV_ALERT_STS [
        ENABLE_FIELD_ALERT OFFSET(0) NUMBITS(1) [],
        SW_APP_ENABLE_FIELD_ALERT OFFSET(1) NUMBITS(1) [],
        READ_INT_STATE_FIELD_ALERT OFFSET(2) NUMBITS(1) [],
    ],
    ERR_CODE [
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
        FIFO_WRITE_ERR OFFSET(28) NUMBITS(1) [],
        FIFO_READ_ERR OFFSET(29) NUMBITS(1) [],
        FIFO_STATE_ERR OFFSET(30) NUMBITS(1) [],
    ],
    ERR_CODE_TEST [
        ERR_CODE_TEST OFFSET(0) NUMBITS(5) [],
    ],
    SEL_TRACKING_SM [
        SEL_TRACKING_SM OFFSET(0) NUMBITS(2) [],
    ],
    TRACKING_SM_OBS [
        TRACKING_SM_OBS0 OFFSET(0) NUMBITS(8) [],
        TRACKING_SM_OBS1 OFFSET(8) NUMBITS(8) [],
        TRACKING_SM_OBS2 OFFSET(16) NUMBITS(8) [],
        TRACKING_SM_OBS3 OFFSET(24) NUMBITS(8) [],
    ],
];

// Number of alerts
pub const CSRNG_PARAM_NUM_ALERTS: u32 = 2;

// Register width
pub const CSRNG_PARAM_REG_WIDTH: u32 = 32;

// End generated register constants for csrng

