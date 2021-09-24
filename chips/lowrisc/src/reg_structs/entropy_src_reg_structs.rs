// Generated register struct for entropy_src

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
    pub Entropy_SrcRegisters {
        (0x0 => intr_state: ReadWrite<u32, INTR_STATE::Register>),
        (0x4 => intr_enable: ReadWrite<u32, INTR_ENABLE::Register>),
        (0x8 => intr_test: WriteOnly<u32, INTR_TEST::Register>),
        (0xc => alert_test: WriteOnly<u32, ALERT_TEST::Register>),
        (0x10 => regwen: ReadWrite<u32, REGWEN::Register>),
        (0x14 => rev: ReadOnly<u32, REV::Register>),
        (0x18 => conf: ReadWrite<u32, CONF::Register>),
        (0x1c => rate: ReadWrite<u32, RATE::Register>),
        (0x20 => entropy_control: ReadWrite<u32, ENTROPY_CONTROL::Register>),
        (0x24 => entropy_data: ReadOnly<u32, ENTROPY_DATA::Register>),
        (0x28 => health_test_windows: ReadWrite<u32, HEALTH_TEST_WINDOWS::Register>),
        (0x2c => repcnt_thresholds: ReadWrite<u32, REPCNT_THRESHOLDS::Register>),
        (0x30 => repcnts_thresholds: ReadWrite<u32, REPCNTS_THRESHOLDS::Register>),
        (0x34 => adaptp_hi_thresholds: ReadWrite<u32, ADAPTP_HI_THRESHOLDS::Register>),
        (0x38 => adaptp_lo_thresholds: ReadWrite<u32, ADAPTP_LO_THRESHOLDS::Register>),
        (0x3c => bucket_thresholds: ReadWrite<u32, BUCKET_THRESHOLDS::Register>),
        (0x40 => markov_hi_thresholds: ReadWrite<u32, MARKOV_HI_THRESHOLDS::Register>),
        (0x44 => markov_lo_thresholds: ReadWrite<u32, MARKOV_LO_THRESHOLDS::Register>),
        (0x48 => extht_hi_thresholds: ReadWrite<u32, EXTHT_HI_THRESHOLDS::Register>),
        (0x4c => extht_lo_thresholds: ReadWrite<u32, EXTHT_LO_THRESHOLDS::Register>),
        (0x50 => repcnt_hi_watermarks: ReadOnly<u32, REPCNT_HI_WATERMARKS::Register>),
        (0x54 => repcnts_hi_watermarks: ReadOnly<u32, REPCNTS_HI_WATERMARKS::Register>),
        (0x58 => adaptp_hi_watermarks: ReadOnly<u32, ADAPTP_HI_WATERMARKS::Register>),
        (0x5c => adaptp_lo_watermarks: ReadOnly<u32, ADAPTP_LO_WATERMARKS::Register>),
        (0x60 => extht_hi_watermarks: ReadOnly<u32, EXTHT_HI_WATERMARKS::Register>),
        (0x64 => extht_lo_watermarks: ReadOnly<u32, EXTHT_LO_WATERMARKS::Register>),
        (0x68 => bucket_hi_watermarks: ReadOnly<u32, BUCKET_HI_WATERMARKS::Register>),
        (0x6c => markov_hi_watermarks: ReadOnly<u32, MARKOV_HI_WATERMARKS::Register>),
        (0x70 => markov_lo_watermarks: ReadOnly<u32, MARKOV_LO_WATERMARKS::Register>),
        (0x74 => repcnt_total_fails: ReadOnly<u32, REPCNT_TOTAL_FAILS::Register>),
        (0x78 => repcnts_total_fails: ReadOnly<u32, REPCNTS_TOTAL_FAILS::Register>),
        (0x7c => adaptp_hi_total_fails: ReadOnly<u32, ADAPTP_HI_TOTAL_FAILS::Register>),
        (0x80 => adaptp_lo_total_fails: ReadOnly<u32, ADAPTP_LO_TOTAL_FAILS::Register>),
        (0x84 => bucket_total_fails: ReadOnly<u32, BUCKET_TOTAL_FAILS::Register>),
        (0x88 => markov_hi_total_fails: ReadOnly<u32, MARKOV_HI_TOTAL_FAILS::Register>),
        (0x8c => markov_lo_total_fails: ReadOnly<u32, MARKOV_LO_TOTAL_FAILS::Register>),
        (0x90 => extht_hi_total_fails: ReadOnly<u32, EXTHT_HI_TOTAL_FAILS::Register>),
        (0x94 => extht_lo_total_fails: ReadOnly<u32, EXTHT_LO_TOTAL_FAILS::Register>),
        (0x98 => alert_threshold: ReadWrite<u32, ALERT_THRESHOLD::Register>),
        (0x9c => alert_summary_fail_counts: ReadOnly<u32, ALERT_SUMMARY_FAIL_COUNTS::Register>),
        (0xa0 => alert_fail_counts: ReadOnly<u32, ALERT_FAIL_COUNTS::Register>),
        (0xa4 => extht_fail_counts: ReadOnly<u32, EXTHT_FAIL_COUNTS::Register>),
        (0xa8 => fw_ov_control: ReadWrite<u32, FW_OV_CONTROL::Register>),
        (0xac => fw_ov_rd_data: ReadOnly<u32, FW_OV_RD_DATA::Register>),
        (0xb0 => fw_ov_wr_data: WriteOnly<u32, FW_OV_WR_DATA::Register>),
        (0xb4 => observe_fifo_thresh: ReadWrite<u32, OBSERVE_FIFO_THRESH::Register>),
        (0xb8 => debug_status: ReadOnly<u32, DEBUG_STATUS::Register>),
        (0xbc => seed: ReadWrite<u32, SEED::Register>),
        (0xc0 => recov_alert_sts: ReadWrite<u32, RECOV_ALERT_STS::Register>),
        (0xc4 => err_code: ReadOnly<u32, ERR_CODE::Register>),
        (0xc8 => err_code_test: ReadWrite<u32, ERR_CODE_TEST::Register>),
    }
}

register_bitfields![u32,
    INTR_STATE [
        ES_ENTROPY_VALID OFFSET(0) NUMBITS(1) [],
        ES_HEALTH_TEST_FAILED OFFSET(1) NUMBITS(1) [],
        ES_OBSERVE_FIFO_READY OFFSET(2) NUMBITS(1) [],
        ES_FATAL_ERR OFFSET(3) NUMBITS(1) [],
    ],
    INTR_ENABLE [
        ES_ENTROPY_VALID OFFSET(0) NUMBITS(1) [],
        ES_HEALTH_TEST_FAILED OFFSET(1) NUMBITS(1) [],
        ES_OBSERVE_FIFO_READY OFFSET(2) NUMBITS(1) [],
        ES_FATAL_ERR OFFSET(3) NUMBITS(1) [],
    ],
    INTR_TEST [
        ES_ENTROPY_VALID OFFSET(0) NUMBITS(1) [],
        ES_HEALTH_TEST_FAILED OFFSET(1) NUMBITS(1) [],
        ES_OBSERVE_FIFO_READY OFFSET(2) NUMBITS(1) [],
        ES_FATAL_ERR OFFSET(3) NUMBITS(1) [],
    ],
    ALERT_TEST [
        RECOV_ALERT OFFSET(0) NUMBITS(1) [],
        FATAL_ALERT OFFSET(1) NUMBITS(1) [],
    ],
    REGWEN [
        REGWEN OFFSET(0) NUMBITS(1) [],
    ],
    REV [
        ABI_REVISION OFFSET(0) NUMBITS(8) [],
        HW_REVISION OFFSET(8) NUMBITS(8) [],
        CHIP_TYPE OFFSET(16) NUMBITS(8) [],
    ],
    CONF [
        ENABLE OFFSET(0) NUMBITS(4) [],
        ENTROPY_DATA_REG_ENABLE OFFSET(4) NUMBITS(4) [],
        LFSR_ENABLE OFFSET(8) NUMBITS(4) [],
        BOOT_BYPASS_DISABLE OFFSET(12) NUMBITS(4) [],
        HEALTH_TEST_CLR OFFSET(16) NUMBITS(4) [],
        RNG_BIT_ENABLE OFFSET(20) NUMBITS(4) [],
        RNG_BIT_SEL OFFSET(24) NUMBITS(2) [],
    ],
    RATE [
        ENTROPY_RATE OFFSET(0) NUMBITS(16) [],
    ],
    ENTROPY_CONTROL [
        ES_ROUTE OFFSET(0) NUMBITS(4) [],
        ES_TYPE OFFSET(4) NUMBITS(4) [],
    ],
    ENTROPY_DATA [
        ENTROPY_DATA OFFSET(0) NUMBITS(32) [],
    ],
    HEALTH_TEST_WINDOWS [
        FIPS_WINDOW OFFSET(0) NUMBITS(16) [],
        BYPASS_WINDOW OFFSET(16) NUMBITS(16) [],
    ],
    REPCNT_THRESHOLDS [
        FIPS_THRESH OFFSET(0) NUMBITS(16) [],
        BYPASS_THRESH OFFSET(16) NUMBITS(16) [],
    ],
    REPCNTS_THRESHOLDS [
        FIPS_THRESH OFFSET(0) NUMBITS(16) [],
        BYPASS_THRESH OFFSET(16) NUMBITS(16) [],
    ],
    ADAPTP_HI_THRESHOLDS [
        FIPS_THRESH OFFSET(0) NUMBITS(16) [],
        BYPASS_THRESH OFFSET(16) NUMBITS(16) [],
    ],
    ADAPTP_LO_THRESHOLDS [
        FIPS_THRESH OFFSET(0) NUMBITS(16) [],
        BYPASS_THRESH OFFSET(16) NUMBITS(16) [],
    ],
    BUCKET_THRESHOLDS [
        FIPS_THRESH OFFSET(0) NUMBITS(16) [],
        BYPASS_THRESH OFFSET(16) NUMBITS(16) [],
    ],
    MARKOV_HI_THRESHOLDS [
        FIPS_THRESH OFFSET(0) NUMBITS(16) [],
        BYPASS_THRESH OFFSET(16) NUMBITS(16) [],
    ],
    MARKOV_LO_THRESHOLDS [
        FIPS_THRESH OFFSET(0) NUMBITS(16) [],
        BYPASS_THRESH OFFSET(16) NUMBITS(16) [],
    ],
    EXTHT_HI_THRESHOLDS [
        FIPS_THRESH OFFSET(0) NUMBITS(16) [],
        BYPASS_THRESH OFFSET(16) NUMBITS(16) [],
    ],
    EXTHT_LO_THRESHOLDS [
        FIPS_THRESH OFFSET(0) NUMBITS(16) [],
        BYPASS_THRESH OFFSET(16) NUMBITS(16) [],
    ],
    REPCNT_HI_WATERMARKS [
        FIPS_WATERMARK OFFSET(0) NUMBITS(16) [],
        BYPASS_WATERMARK OFFSET(16) NUMBITS(16) [],
    ],
    REPCNTS_HI_WATERMARKS [
        FIPS_WATERMARK OFFSET(0) NUMBITS(16) [],
        BYPASS_WATERMARK OFFSET(16) NUMBITS(16) [],
    ],
    ADAPTP_HI_WATERMARKS [
        FIPS_WATERMARK OFFSET(0) NUMBITS(16) [],
        BYPASS_WATERMARK OFFSET(16) NUMBITS(16) [],
    ],
    ADAPTP_LO_WATERMARKS [
        FIPS_WATERMARK OFFSET(0) NUMBITS(16) [],
        BYPASS_WATERMARK OFFSET(16) NUMBITS(16) [],
    ],
    EXTHT_HI_WATERMARKS [
        FIPS_WATERMARK OFFSET(0) NUMBITS(16) [],
        BYPASS_WATERMARK OFFSET(16) NUMBITS(16) [],
    ],
    EXTHT_LO_WATERMARKS [
        FIPS_WATERMARK OFFSET(0) NUMBITS(16) [],
        BYPASS_WATERMARK OFFSET(16) NUMBITS(16) [],
    ],
    BUCKET_HI_WATERMARKS [
        FIPS_WATERMARK OFFSET(0) NUMBITS(16) [],
        BYPASS_WATERMARK OFFSET(16) NUMBITS(16) [],
    ],
    MARKOV_HI_WATERMARKS [
        FIPS_WATERMARK OFFSET(0) NUMBITS(16) [],
        BYPASS_WATERMARK OFFSET(16) NUMBITS(16) [],
    ],
    MARKOV_LO_WATERMARKS [
        FIPS_WATERMARK OFFSET(0) NUMBITS(16) [],
        BYPASS_WATERMARK OFFSET(16) NUMBITS(16) [],
    ],
    REPCNT_TOTAL_FAILS [
        REPCNT_TOTAL_FAILS OFFSET(0) NUMBITS(32) [],
    ],
    REPCNTS_TOTAL_FAILS [
        REPCNTS_TOTAL_FAILS OFFSET(0) NUMBITS(32) [],
    ],
    ADAPTP_HI_TOTAL_FAILS [
        ADAPTP_HI_TOTAL_FAILS OFFSET(0) NUMBITS(32) [],
    ],
    ADAPTP_LO_TOTAL_FAILS [
        ADAPTP_LO_TOTAL_FAILS OFFSET(0) NUMBITS(32) [],
    ],
    BUCKET_TOTAL_FAILS [
        BUCKET_TOTAL_FAILS OFFSET(0) NUMBITS(32) [],
    ],
    MARKOV_HI_TOTAL_FAILS [
        MARKOV_HI_TOTAL_FAILS OFFSET(0) NUMBITS(32) [],
    ],
    MARKOV_LO_TOTAL_FAILS [
        MARKOV_LO_TOTAL_FAILS OFFSET(0) NUMBITS(32) [],
    ],
    EXTHT_HI_TOTAL_FAILS [
        EXTHT_HI_TOTAL_FAILS OFFSET(0) NUMBITS(32) [],
    ],
    EXTHT_LO_TOTAL_FAILS [
        EXTHT_LO_TOTAL_FAILS OFFSET(0) NUMBITS(32) [],
    ],
    ALERT_THRESHOLD [
        ALERT_THRESHOLD OFFSET(0) NUMBITS(16) [],
        ALERT_THRESHOLD_INV OFFSET(16) NUMBITS(16) [],
    ],
    ALERT_SUMMARY_FAIL_COUNTS [
        ANY_FAIL_COUNT OFFSET(0) NUMBITS(16) [],
    ],
    ALERT_FAIL_COUNTS [
        REPCNT_FAIL_COUNT OFFSET(4) NUMBITS(4) [],
        ADAPTP_HI_FAIL_COUNT OFFSET(8) NUMBITS(4) [],
        ADAPTP_LO_FAIL_COUNT OFFSET(12) NUMBITS(4) [],
        BUCKET_FAIL_COUNT OFFSET(16) NUMBITS(4) [],
        MARKOV_HI_FAIL_COUNT OFFSET(20) NUMBITS(4) [],
        MARKOV_LO_FAIL_COUNT OFFSET(24) NUMBITS(4) [],
        REPCNTS_FAIL_COUNT OFFSET(28) NUMBITS(4) [],
    ],
    EXTHT_FAIL_COUNTS [
        EXTHT_HI_FAIL_COUNT OFFSET(0) NUMBITS(4) [],
        EXTHT_LO_FAIL_COUNT OFFSET(4) NUMBITS(4) [],
    ],
    FW_OV_CONTROL [
        FW_OV_MODE OFFSET(0) NUMBITS(4) [],
        FW_OV_ENTROPY_INSERT OFFSET(4) NUMBITS(4) [],
    ],
    FW_OV_RD_DATA [
        FW_OV_RD_DATA OFFSET(0) NUMBITS(32) [],
    ],
    FW_OV_WR_DATA [
        FW_OV_WR_DATA OFFSET(0) NUMBITS(32) [],
    ],
    OBSERVE_FIFO_THRESH [
        OBSERVE_FIFO_THRESH OFFSET(0) NUMBITS(7) [],
    ],
    DEBUG_STATUS [
        ENTROPY_FIFO_DEPTH OFFSET(0) NUMBITS(3) [],
        SHA3_FSM OFFSET(3) NUMBITS(3) [],
        SHA3_BLOCK_PR OFFSET(6) NUMBITS(1) [],
        SHA3_SQUEEZING OFFSET(7) NUMBITS(1) [],
        SHA3_ABSORBED OFFSET(8) NUMBITS(1) [],
        SHA3_ERR OFFSET(9) NUMBITS(1) [],
        MAIN_SM_IDLE OFFSET(16) NUMBITS(1) [],
        MAIN_SM_STATE OFFSET(24) NUMBITS(8) [],
    ],
    SEED [
        LFSR_SEED OFFSET(0) NUMBITS(4) [],
    ],
    RECOV_ALERT_STS [
        ES_MAIN_SM_ALERT OFFSET(12) NUMBITS(1) [],
        ES_BUS_CMP_ALERT OFFSET(13) NUMBITS(1) [],
    ],
    ERR_CODE [
        SFIFO_ESRNG_ERR OFFSET(0) NUMBITS(1) [],
        SFIFO_OBSERVE_ERR OFFSET(1) NUMBITS(1) [],
        SFIFO_ESFINAL_ERR OFFSET(2) NUMBITS(1) [],
        ES_ACK_SM_ERR OFFSET(20) NUMBITS(1) [],
        ES_MAIN_SM_ERR OFFSET(21) NUMBITS(1) [],
        FIFO_WRITE_ERR OFFSET(28) NUMBITS(1) [],
        FIFO_READ_ERR OFFSET(29) NUMBITS(1) [],
        FIFO_STATE_ERR OFFSET(30) NUMBITS(1) [],
    ],
    ERR_CODE_TEST [
        ERR_CODE_TEST OFFSET(0) NUMBITS(5) [],
    ],
];

// Number of alerts
pub const ENTROPY_SRC_PARAM_NUM_ALERTS: u32 = 2;

// Register width
pub const ENTROPY_SRC_PARAM_REG_WIDTH: u32 = 32;

// End generated register constants for entropy_src

