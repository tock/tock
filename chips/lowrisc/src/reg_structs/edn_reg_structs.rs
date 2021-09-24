// Generated register struct for edn

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
    pub EdnRegisters {
        (0x0 => intr_state: ReadWrite<u32, INTR_STATE::Register>),
        (0x4 => intr_enable: ReadWrite<u32, INTR_ENABLE::Register>),
        (0x8 => intr_test: WriteOnly<u32, INTR_TEST::Register>),
        (0xc => alert_test: WriteOnly<u32, ALERT_TEST::Register>),
        (0x10 => regwen: ReadWrite<u32, REGWEN::Register>),
        (0x14 => ctrl: ReadWrite<u32, CTRL::Register>),
        (0x18 => sum_sts: ReadWrite<u32, SUM_STS::Register>),
        (0x1c => sw_cmd_req: WriteOnly<u32, SW_CMD_REQ::Register>),
        (0x20 => sw_cmd_sts: ReadOnly<u32, SW_CMD_STS::Register>),
        (0x24 => reseed_cmd: WriteOnly<u32, RESEED_CMD::Register>),
        (0x28 => generate_cmd: WriteOnly<u32, GENERATE_CMD::Register>),
        (0x2c => max_num_reqs_between_reseeds: ReadWrite<u32, MAX_NUM_REQS_BETWEEN_RESEEDS::Register>),
        (0x30 => recov_alert_sts: ReadWrite<u32, RECOV_ALERT_STS::Register>),
        (0x34 => err_code: ReadOnly<u32, ERR_CODE::Register>),
        (0x38 => err_code_test: ReadWrite<u32, ERR_CODE_TEST::Register>),
    }
}

register_bitfields![u32,
    INTR_STATE [
        EDN_CMD_REQ_DONE OFFSET(0) NUMBITS(1) [],
        EDN_FATAL_ERR OFFSET(1) NUMBITS(1) [],
    ],
    INTR_ENABLE [
        EDN_CMD_REQ_DONE OFFSET(0) NUMBITS(1) [],
        EDN_FATAL_ERR OFFSET(1) NUMBITS(1) [],
    ],
    INTR_TEST [
        EDN_CMD_REQ_DONE OFFSET(0) NUMBITS(1) [],
        EDN_FATAL_ERR OFFSET(1) NUMBITS(1) [],
    ],
    ALERT_TEST [
        RECOV_ALERT OFFSET(0) NUMBITS(1) [],
        FATAL_ALERT OFFSET(1) NUMBITS(1) [],
    ],
    REGWEN [
        REGWEN OFFSET(0) NUMBITS(1) [],
    ],
    CTRL [
        EDN_ENABLE OFFSET(0) NUMBITS(4) [],
        BOOT_REQ_MODE OFFSET(4) NUMBITS(4) [],
        AUTO_REQ_MODE OFFSET(8) NUMBITS(4) [],
        CMD_FIFO_RST OFFSET(12) NUMBITS(4) [],
    ],
    SUM_STS [
        REQ_MODE_SM_STS OFFSET(0) NUMBITS(1) [],
        BOOT_INST_ACK OFFSET(1) NUMBITS(1) [],
    ],
    SW_CMD_REQ [
        SW_CMD_REQ OFFSET(0) NUMBITS(32) [],
    ],
    SW_CMD_STS [
        CMD_RDY OFFSET(0) NUMBITS(1) [],
        CMD_STS OFFSET(1) NUMBITS(1) [],
    ],
    RESEED_CMD [
        RESEED_CMD OFFSET(0) NUMBITS(32) [],
    ],
    GENERATE_CMD [
        GENERATE_CMD OFFSET(0) NUMBITS(32) [],
    ],
    MAX_NUM_REQS_BETWEEN_RESEEDS [
        MAX_NUM_REQS_BETWEEN_RESEEDS OFFSET(0) NUMBITS(32) [],
    ],
    RECOV_ALERT_STS [
        EDN_ENABLE_FIELD_ALERT OFFSET(0) NUMBITS(1) [],
        BOOT_REQ_MODE_FIELD_ALERT OFFSET(1) NUMBITS(1) [],
        AUTO_REQ_MODE_FIELD_ALERT OFFSET(2) NUMBITS(1) [],
        CMD_FIFO_RST_FIELD_ALERT OFFSET(3) NUMBITS(1) [],
        EDN_BUS_CMP_ALERT OFFSET(12) NUMBITS(1) [],
    ],
    ERR_CODE [
        SFIFO_RESCMD_ERR OFFSET(0) NUMBITS(1) [],
        SFIFO_GENCMD_ERR OFFSET(1) NUMBITS(1) [],
        EDN_ACK_SM_ERR OFFSET(20) NUMBITS(1) [],
        EDN_MAIN_SM_ERR OFFSET(21) NUMBITS(1) [],
        FIFO_WRITE_ERR OFFSET(28) NUMBITS(1) [],
        FIFO_READ_ERR OFFSET(29) NUMBITS(1) [],
        FIFO_STATE_ERR OFFSET(30) NUMBITS(1) [],
    ],
    ERR_CODE_TEST [
        ERR_CODE_TEST OFFSET(0) NUMBITS(5) [],
    ],
];

// Number of alerts
pub const EDN_PARAM_NUM_ALERTS: u32 = 2;

// Register width
pub const EDN_PARAM_REG_WIDTH: u32 = 32;

// End generated register constants for edn

