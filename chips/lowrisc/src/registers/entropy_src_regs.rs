// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright lowRISC contributors 2023.

// Generated register constants for entropy_src.
// Built for Earlgrey-M2.5.1-RC1-438-gacc67de99
// https://github.com/lowRISC/opentitan/tree/acc67de992ee8de5f2481b1b9580679850d8b5f5
// Tree status: clean
// Build date: 2023-08-08T00:15:38

// Original reference file: hw/ip/entropy_src/data/entropy_src.hjson
use kernel::utilities::registers::ReadWrite;
use kernel::utilities::registers::{register_bitfields, register_structs};
/// Number of alerts
pub const ENTROPY_SRC_PARAM_NUM_ALERTS: u32 = 2;
/// Register width
pub const ENTROPY_SRC_PARAM_REG_WIDTH: u32 = 32;

register_structs! {
    pub EntropySrcRegisters {
        /// Interrupt State Register
        (0x0000 => pub(crate) intr_state: ReadWrite<u32, INTR::Register>),
        /// Interrupt Enable Register
        (0x0004 => pub(crate) intr_enable: ReadWrite<u32, INTR::Register>),
        /// Interrupt Test Register
        (0x0008 => pub(crate) intr_test: ReadWrite<u32, INTR::Register>),
        /// Alert Test Register
        (0x000c => pub(crate) alert_test: ReadWrite<u32, ALERT_TEST::Register>),
        /// Register write enable for module enable register
        (0x0010 => pub(crate) me_regwen: ReadWrite<u32, ME_REGWEN::Register>),
        /// Register write enable for control and threshold registers
        (0x0014 => pub(crate) sw_regupd: ReadWrite<u32, SW_REGUPD::Register>),
        /// Register write enable for all control registers
        (0x0018 => pub(crate) regwen: ReadWrite<u32, REGWEN::Register>),
        /// Revision register
        (0x001c => pub(crate) rev: ReadWrite<u32, REV::Register>),
        /// Module enable register
        (0x0020 => pub(crate) module_enable: ReadWrite<u32, MODULE_ENABLE::Register>),
        /// Configuration register
        (0x0024 => pub(crate) conf: ReadWrite<u32, CONF::Register>),
        /// Entropy control register
        (0x0028 => pub(crate) entropy_control: ReadWrite<u32, ENTROPY_CONTROL::Register>),
        /// Entropy data bits
        (0x002c => pub(crate) entropy_data: ReadWrite<u32, ENTROPY_DATA::Register>),
        /// Health test windows register
        (0x0030 => pub(crate) health_test_windows: ReadWrite<u32, HEALTH_TEST_WINDOWS::Register>),
        /// Repetition count test thresholds register
        (0x0034 => pub(crate) repcnt_thresholds: ReadWrite<u32, REPCNT_THRESHOLDS::Register>),
        /// Repetition count symbol test thresholds register
        (0x0038 => pub(crate) repcnts_thresholds: ReadWrite<u32, REPCNTS_THRESHOLDS::Register>),
        /// Adaptive proportion test high thresholds register
        (0x003c => pub(crate) adaptp_hi_thresholds: ReadWrite<u32, ADAPTP_HI_THRESHOLDS::Register>),
        /// Adaptive proportion test low thresholds register
        (0x0040 => pub(crate) adaptp_lo_thresholds: ReadWrite<u32, ADAPTP_LO_THRESHOLDS::Register>),
        /// Bucket test thresholds register
        (0x0044 => pub(crate) bucket_thresholds: ReadWrite<u32, BUCKET_THRESHOLDS::Register>),
        /// Markov test high thresholds register
        (0x0048 => pub(crate) markov_hi_thresholds: ReadWrite<u32, MARKOV_HI_THRESHOLDS::Register>),
        /// Markov test low thresholds register
        (0x004c => pub(crate) markov_lo_thresholds: ReadWrite<u32, MARKOV_LO_THRESHOLDS::Register>),
        /// External health test high thresholds register
        (0x0050 => pub(crate) extht_hi_thresholds: ReadWrite<u32, EXTHT_HI_THRESHOLDS::Register>),
        /// External health test low thresholds register
        (0x0054 => pub(crate) extht_lo_thresholds: ReadWrite<u32, EXTHT_LO_THRESHOLDS::Register>),
        /// Repetition count test high watermarks register
        (0x0058 => pub(crate) repcnt_hi_watermarks: ReadWrite<u32, REPCNT_HI_WATERMARKS::Register>),
        /// Repetition count symbol test high watermarks register
        (0x005c => pub(crate) repcnts_hi_watermarks: ReadWrite<u32, REPCNTS_HI_WATERMARKS::Register>),
        /// Adaptive proportion test high watermarks register
        (0x0060 => pub(crate) adaptp_hi_watermarks: ReadWrite<u32, ADAPTP_HI_WATERMARKS::Register>),
        /// Adaptive proportion test low watermarks register
        (0x0064 => pub(crate) adaptp_lo_watermarks: ReadWrite<u32, ADAPTP_LO_WATERMARKS::Register>),
        /// External health test high watermarks register
        (0x0068 => pub(crate) extht_hi_watermarks: ReadWrite<u32, EXTHT_HI_WATERMARKS::Register>),
        /// External health test low watermarks register
        (0x006c => pub(crate) extht_lo_watermarks: ReadWrite<u32, EXTHT_LO_WATERMARKS::Register>),
        /// Bucket test high watermarks register
        (0x0070 => pub(crate) bucket_hi_watermarks: ReadWrite<u32, BUCKET_HI_WATERMARKS::Register>),
        /// Markov test high watermarks register
        (0x0074 => pub(crate) markov_hi_watermarks: ReadWrite<u32, MARKOV_HI_WATERMARKS::Register>),
        /// Markov test low watermarks register
        (0x0078 => pub(crate) markov_lo_watermarks: ReadWrite<u32, MARKOV_LO_WATERMARKS::Register>),
        /// Repetition count test failure counter register
        (0x007c => pub(crate) repcnt_total_fails: ReadWrite<u32, REPCNT_TOTAL_FAILS::Register>),
        /// Repetition count symbol test failure counter register
        (0x0080 => pub(crate) repcnts_total_fails: ReadWrite<u32, REPCNTS_TOTAL_FAILS::Register>),
        /// Adaptive proportion high test failure counter register
        (0x0084 => pub(crate) adaptp_hi_total_fails: ReadWrite<u32, ADAPTP_HI_TOTAL_FAILS::Register>),
        /// Adaptive proportion low test failure counter register
        (0x0088 => pub(crate) adaptp_lo_total_fails: ReadWrite<u32, ADAPTP_LO_TOTAL_FAILS::Register>),
        /// Bucket test failure counter register
        (0x008c => pub(crate) bucket_total_fails: ReadWrite<u32, BUCKET_TOTAL_FAILS::Register>),
        /// Markov high test failure counter register
        (0x0090 => pub(crate) markov_hi_total_fails: ReadWrite<u32, MARKOV_HI_TOTAL_FAILS::Register>),
        /// Markov low test failure counter register
        (0x0094 => pub(crate) markov_lo_total_fails: ReadWrite<u32, MARKOV_LO_TOTAL_FAILS::Register>),
        /// External health test high threshold failure counter register
        (0x0098 => pub(crate) extht_hi_total_fails: ReadWrite<u32, EXTHT_HI_TOTAL_FAILS::Register>),
        /// External health test low threshold failure counter register
        (0x009c => pub(crate) extht_lo_total_fails: ReadWrite<u32, EXTHT_LO_TOTAL_FAILS::Register>),
        /// Alert threshold register
        (0x00a0 => pub(crate) alert_threshold: ReadWrite<u32, ALERT_THRESHOLD::Register>),
        /// Alert summary failure counts register
        (0x00a4 => pub(crate) alert_summary_fail_counts: ReadWrite<u32, ALERT_SUMMARY_FAIL_COUNTS::Register>),
        /// Alert failure counts register
        (0x00a8 => pub(crate) alert_fail_counts: ReadWrite<u32, ALERT_FAIL_COUNTS::Register>),
        /// External health test alert failure counts register
        (0x00ac => pub(crate) extht_fail_counts: ReadWrite<u32, EXTHT_FAIL_COUNTS::Register>),
        /// Firmware override control register
        (0x00b0 => pub(crate) fw_ov_control: ReadWrite<u32, FW_OV_CONTROL::Register>),
        /// Firmware override sha3 block start control register
        (0x00b4 => pub(crate) fw_ov_sha3_start: ReadWrite<u32, FW_OV_SHA3_START::Register>),
        /// Firmware override FIFO write full status register
        (0x00b8 => pub(crate) fw_ov_wr_fifo_full: ReadWrite<u32, FW_OV_WR_FIFO_FULL::Register>),
        /// Firmware override observe FIFO overflow status
        (0x00bc => pub(crate) fw_ov_rd_fifo_overflow: ReadWrite<u32, FW_OV_RD_FIFO_OVERFLOW::Register>),
        /// Firmware override observe FIFO read register
        (0x00c0 => pub(crate) fw_ov_rd_data: ReadWrite<u32, FW_OV_RD_DATA::Register>),
        /// Firmware override FIFO write register
        (0x00c4 => pub(crate) fw_ov_wr_data: ReadWrite<u32, FW_OV_WR_DATA::Register>),
        /// Observe FIFO threshold register
        (0x00c8 => pub(crate) observe_fifo_thresh: ReadWrite<u32, OBSERVE_FIFO_THRESH::Register>),
        /// Observe FIFO depth register
        (0x00cc => pub(crate) observe_fifo_depth: ReadWrite<u32, OBSERVE_FIFO_DEPTH::Register>),
        /// Debug status register
        (0x00d0 => pub(crate) debug_status: ReadWrite<u32, DEBUG_STATUS::Register>),
        /// Recoverable alert status register
        (0x00d4 => pub(crate) recov_alert_sts: ReadWrite<u32, RECOV_ALERT_STS::Register>),
        /// Hardware detection of error conditions status register
        (0x00d8 => pub(crate) err_code: ReadWrite<u32, ERR_CODE::Register>),
        /// Test error conditions register
        (0x00dc => pub(crate) err_code_test: ReadWrite<u32, ERR_CODE_TEST::Register>),
        /// Main state machine state debug register
        (0x00e0 => pub(crate) main_sm_state: ReadWrite<u32, MAIN_SM_STATE::Register>),
        (0x00e4 => @END),
    }
}

register_bitfields![u32,
    /// Common Interrupt Offsets
    pub(crate) INTR [
        ES_ENTROPY_VALID OFFSET(0) NUMBITS(1) [],
        ES_HEALTH_TEST_FAILED OFFSET(1) NUMBITS(1) [],
        ES_OBSERVE_FIFO_READY OFFSET(2) NUMBITS(1) [],
        ES_FATAL_ERR OFFSET(3) NUMBITS(1) [],
    ],
    pub(crate) ALERT_TEST [
        RECOV_ALERT OFFSET(0) NUMBITS(1) [],
        FATAL_ALERT OFFSET(1) NUMBITS(1) [],
    ],
    pub(crate) ME_REGWEN [
        ME_REGWEN OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) SW_REGUPD [
        SW_REGUPD OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) REGWEN [
        REGWEN OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) REV [
        ABI_REVISION OFFSET(0) NUMBITS(8) [],
        HW_REVISION OFFSET(8) NUMBITS(8) [],
        CHIP_TYPE OFFSET(16) NUMBITS(8) [],
    ],
    pub(crate) MODULE_ENABLE [
        MODULE_ENABLE OFFSET(0) NUMBITS(4) [],
    ],
    pub(crate) CONF [
        FIPS_ENABLE OFFSET(0) NUMBITS(4) [],
        ENTROPY_DATA_REG_ENABLE OFFSET(4) NUMBITS(4) [],
        THRESHOLD_SCOPE OFFSET(12) NUMBITS(4) [],
        RNG_BIT_ENABLE OFFSET(20) NUMBITS(4) [],
        RNG_BIT_SEL OFFSET(24) NUMBITS(2) [],
    ],
    pub(crate) ENTROPY_CONTROL [
        ES_ROUTE OFFSET(0) NUMBITS(4) [],
        ES_TYPE OFFSET(4) NUMBITS(4) [],
    ],
    pub(crate) ENTROPY_DATA [
        ENTROPY_DATA OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) HEALTH_TEST_WINDOWS [
        FIPS_WINDOW OFFSET(0) NUMBITS(16) [],
        BYPASS_WINDOW OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) REPCNT_THRESHOLDS [
        FIPS_THRESH OFFSET(0) NUMBITS(16) [],
        BYPASS_THRESH OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) REPCNTS_THRESHOLDS [
        FIPS_THRESH OFFSET(0) NUMBITS(16) [],
        BYPASS_THRESH OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) ADAPTP_HI_THRESHOLDS [
        FIPS_THRESH OFFSET(0) NUMBITS(16) [],
        BYPASS_THRESH OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) ADAPTP_LO_THRESHOLDS [
        FIPS_THRESH OFFSET(0) NUMBITS(16) [],
        BYPASS_THRESH OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) BUCKET_THRESHOLDS [
        FIPS_THRESH OFFSET(0) NUMBITS(16) [],
        BYPASS_THRESH OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) MARKOV_HI_THRESHOLDS [
        FIPS_THRESH OFFSET(0) NUMBITS(16) [],
        BYPASS_THRESH OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) MARKOV_LO_THRESHOLDS [
        FIPS_THRESH OFFSET(0) NUMBITS(16) [],
        BYPASS_THRESH OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) EXTHT_HI_THRESHOLDS [
        FIPS_THRESH OFFSET(0) NUMBITS(16) [],
        BYPASS_THRESH OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) EXTHT_LO_THRESHOLDS [
        FIPS_THRESH OFFSET(0) NUMBITS(16) [],
        BYPASS_THRESH OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) REPCNT_HI_WATERMARKS [
        FIPS_WATERMARK OFFSET(0) NUMBITS(16) [],
        BYPASS_WATERMARK OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) REPCNTS_HI_WATERMARKS [
        FIPS_WATERMARK OFFSET(0) NUMBITS(16) [],
        BYPASS_WATERMARK OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) ADAPTP_HI_WATERMARKS [
        FIPS_WATERMARK OFFSET(0) NUMBITS(16) [],
        BYPASS_WATERMARK OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) ADAPTP_LO_WATERMARKS [
        FIPS_WATERMARK OFFSET(0) NUMBITS(16) [],
        BYPASS_WATERMARK OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) EXTHT_HI_WATERMARKS [
        FIPS_WATERMARK OFFSET(0) NUMBITS(16) [],
        BYPASS_WATERMARK OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) EXTHT_LO_WATERMARKS [
        FIPS_WATERMARK OFFSET(0) NUMBITS(16) [],
        BYPASS_WATERMARK OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) BUCKET_HI_WATERMARKS [
        FIPS_WATERMARK OFFSET(0) NUMBITS(16) [],
        BYPASS_WATERMARK OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) MARKOV_HI_WATERMARKS [
        FIPS_WATERMARK OFFSET(0) NUMBITS(16) [],
        BYPASS_WATERMARK OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) MARKOV_LO_WATERMARKS [
        FIPS_WATERMARK OFFSET(0) NUMBITS(16) [],
        BYPASS_WATERMARK OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) REPCNT_TOTAL_FAILS [
        REPCNT_TOTAL_FAILS OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) REPCNTS_TOTAL_FAILS [
        REPCNTS_TOTAL_FAILS OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) ADAPTP_HI_TOTAL_FAILS [
        ADAPTP_HI_TOTAL_FAILS OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) ADAPTP_LO_TOTAL_FAILS [
        ADAPTP_LO_TOTAL_FAILS OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) BUCKET_TOTAL_FAILS [
        BUCKET_TOTAL_FAILS OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) MARKOV_HI_TOTAL_FAILS [
        MARKOV_HI_TOTAL_FAILS OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) MARKOV_LO_TOTAL_FAILS [
        MARKOV_LO_TOTAL_FAILS OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) EXTHT_HI_TOTAL_FAILS [
        EXTHT_HI_TOTAL_FAILS OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) EXTHT_LO_TOTAL_FAILS [
        EXTHT_LO_TOTAL_FAILS OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) ALERT_THRESHOLD [
        ALERT_THRESHOLD OFFSET(0) NUMBITS(16) [],
        ALERT_THRESHOLD_INV OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) ALERT_SUMMARY_FAIL_COUNTS [
        ANY_FAIL_COUNT OFFSET(0) NUMBITS(16) [],
    ],
    pub(crate) ALERT_FAIL_COUNTS [
        REPCNT_FAIL_COUNT OFFSET(4) NUMBITS(4) [],
        ADAPTP_HI_FAIL_COUNT OFFSET(8) NUMBITS(4) [],
        ADAPTP_LO_FAIL_COUNT OFFSET(12) NUMBITS(4) [],
        BUCKET_FAIL_COUNT OFFSET(16) NUMBITS(4) [],
        MARKOV_HI_FAIL_COUNT OFFSET(20) NUMBITS(4) [],
        MARKOV_LO_FAIL_COUNT OFFSET(24) NUMBITS(4) [],
        REPCNTS_FAIL_COUNT OFFSET(28) NUMBITS(4) [],
    ],
    pub(crate) EXTHT_FAIL_COUNTS [
        EXTHT_HI_FAIL_COUNT OFFSET(0) NUMBITS(4) [],
        EXTHT_LO_FAIL_COUNT OFFSET(4) NUMBITS(4) [],
    ],
    pub(crate) FW_OV_CONTROL [
        FW_OV_MODE OFFSET(0) NUMBITS(4) [],
        FW_OV_ENTROPY_INSERT OFFSET(4) NUMBITS(4) [],
    ],
    pub(crate) FW_OV_SHA3_START [
        FW_OV_INSERT_START OFFSET(0) NUMBITS(4) [],
    ],
    pub(crate) FW_OV_WR_FIFO_FULL [
        FW_OV_WR_FIFO_FULL OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) FW_OV_RD_FIFO_OVERFLOW [
        FW_OV_RD_FIFO_OVERFLOW OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) FW_OV_RD_DATA [
        FW_OV_RD_DATA OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) FW_OV_WR_DATA [
        FW_OV_WR_DATA OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) OBSERVE_FIFO_THRESH [
        OBSERVE_FIFO_THRESH OFFSET(0) NUMBITS(7) [],
    ],
    pub(crate) OBSERVE_FIFO_DEPTH [
        OBSERVE_FIFO_DEPTH OFFSET(0) NUMBITS(7) [],
    ],
    pub(crate) DEBUG_STATUS [
        ENTROPY_FIFO_DEPTH OFFSET(0) NUMBITS(3) [],
        SHA3_FSM OFFSET(3) NUMBITS(3) [],
        SHA3_BLOCK_PR OFFSET(6) NUMBITS(1) [],
        SHA3_SQUEEZING OFFSET(7) NUMBITS(1) [],
        SHA3_ABSORBED OFFSET(8) NUMBITS(1) [],
        SHA3_ERR OFFSET(9) NUMBITS(1) [],
        MAIN_SM_IDLE OFFSET(16) NUMBITS(1) [],
        MAIN_SM_BOOT_DONE OFFSET(17) NUMBITS(1) [],
    ],
    pub(crate) RECOV_ALERT_STS [
        FIPS_ENABLE_FIELD_ALERT OFFSET(0) NUMBITS(1) [],
        ENTROPY_DATA_REG_EN_FIELD_ALERT OFFSET(1) NUMBITS(1) [],
        MODULE_ENABLE_FIELD_ALERT OFFSET(2) NUMBITS(1) [],
        THRESHOLD_SCOPE_FIELD_ALERT OFFSET(3) NUMBITS(1) [],
        RNG_BIT_ENABLE_FIELD_ALERT OFFSET(5) NUMBITS(1) [],
        FW_OV_SHA3_START_FIELD_ALERT OFFSET(7) NUMBITS(1) [],
        FW_OV_MODE_FIELD_ALERT OFFSET(8) NUMBITS(1) [],
        FW_OV_ENTROPY_INSERT_FIELD_ALERT OFFSET(9) NUMBITS(1) [],
        ES_ROUTE_FIELD_ALERT OFFSET(10) NUMBITS(1) [],
        ES_TYPE_FIELD_ALERT OFFSET(11) NUMBITS(1) [],
        ES_MAIN_SM_ALERT OFFSET(12) NUMBITS(1) [],
        ES_BUS_CMP_ALERT OFFSET(13) NUMBITS(1) [],
        ES_THRESH_CFG_ALERT OFFSET(14) NUMBITS(1) [],
        ES_FW_OV_WR_ALERT OFFSET(15) NUMBITS(1) [],
        ES_FW_OV_DISABLE_ALERT OFFSET(16) NUMBITS(1) [],
    ],
    pub(crate) ERR_CODE [
        SFIFO_ESRNG_ERR OFFSET(0) NUMBITS(1) [],
        SFIFO_OBSERVE_ERR OFFSET(1) NUMBITS(1) [],
        SFIFO_ESFINAL_ERR OFFSET(2) NUMBITS(1) [],
        ES_ACK_SM_ERR OFFSET(20) NUMBITS(1) [],
        ES_MAIN_SM_ERR OFFSET(21) NUMBITS(1) [],
        ES_CNTR_ERR OFFSET(22) NUMBITS(1) [],
        SHA3_STATE_ERR OFFSET(23) NUMBITS(1) [],
        SHA3_RST_STORAGE_ERR OFFSET(24) NUMBITS(1) [],
        FIFO_WRITE_ERR OFFSET(28) NUMBITS(1) [],
        FIFO_READ_ERR OFFSET(29) NUMBITS(1) [],
        FIFO_STATE_ERR OFFSET(30) NUMBITS(1) [],
    ],
    pub(crate) ERR_CODE_TEST [
        ERR_CODE_TEST OFFSET(0) NUMBITS(5) [],
    ],
    pub(crate) MAIN_SM_STATE [
        MAIN_SM_STATE OFFSET(0) NUMBITS(9) [],
    ],
];

// End generated register constants for entropy_src
