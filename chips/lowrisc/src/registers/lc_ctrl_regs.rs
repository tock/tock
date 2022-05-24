// Generated register constants for lc_ctrl.
// This file is licensed under either of:
//   Apache License, Version 2.0 (LICENSE-APACHE <http://www.apache.org/licenses/LICENSE-2.0>)
//   MIT License (LICENSE-MIT <http://opensource.org/licenses/MIT>)

// Built for earlgrey_silver_release_v5-5654-g222658011
// https://github.com/lowRISC/opentitan/tree/222658011c27d6c1f22f02c7f589043f207ff574
// Tree status: clean
// Build date: 2022-06-02T20:40:57

// Original reference file: hw/ip/lc_ctrl/data/lc_ctrl.hjson
// Copyright information found in the reference file:
//   Copyright lowRISC contributors.
// Licensing information found in the reference file:
//   Licensed under the Apache License, Version 2.0, see LICENSE for details.
//   SPDX-License-Identifier: Apache-2.0

use kernel::utilities::registers::ReadWrite;
use kernel::utilities::registers::{register_bitfields, register_structs};
// Width of hardware revision fields.
pub const LC_CTRL_PARAM_HW_REV_FIELD_WIDTH: u32 = 16;
// Number of 32bit words in a token.
pub const LC_CTRL_PARAM_NUM_TOKEN_WORDS: u32 = 4;
// Number of life cycle state enum bits.
pub const LC_CTRL_PARAM_CSR_LC_STATE_WIDTH: u32 = 30;
// Number of life cycle transition counter bits.
pub const LC_CTRL_PARAM_CSR_LC_COUNT_WIDTH: u32 = 5;
// Number of life cycle id state enum bits.
pub const LC_CTRL_PARAM_CSR_LC_ID_STATE_WIDTH: u32 = 32;
// Number of vendor/test-specific OTP control bits.
pub const LC_CTRL_PARAM_CSR_OTP_TEST_CTRL_WIDTH: u32 = 32;
// Number of vendor/test-specific OTP status bits.
pub const LC_CTRL_PARAM_CSR_OTP_TEST_STATUS_WIDTH: u32 = 32;
// Number of 32bit words in the Device ID.
pub const LC_CTRL_PARAM_NUM_DEVICE_ID_WORDS: u32 = 8;
// Number of 32bit words in the manufacturing state.
pub const LC_CTRL_PARAM_NUM_MANUF_STATE_WORDS: u32 = 8;
// Number of alerts
pub const LC_CTRL_PARAM_NUM_ALERTS: u32 = 3;
// Register width
pub const LC_CTRL_PARAM_REG_WIDTH: u32 = 32;

register_structs! {
    pub LcCtrlRegisters {
        // Alert Test Register
        (0x0000 => pub(crate) alert_test: ReadWrite<u32, ALERT_TEST::Register>),
        // life cycle status register. Note that all errors are terminal and require a reset cycle.
        (0x0004 => pub(crate) status: ReadWrite<u32, STATUS::Register>),
        // Hardware mutex to claim exclusive access to the transition interface.
        (0x0008 => pub(crate) claim_transition_if: ReadWrite<u32, CLAIM_TRANSITION_IF::Register>),
        // Register write enable for all transition interface registers.
        (0x000c => pub(crate) transition_regwen: ReadWrite<u32, TRANSITION_REGWEN::Register>),
        // Command register for state transition requests.
        (0x0010 => pub(crate) transition_cmd: ReadWrite<u32, TRANSITION_CMD::Register>),
        // Control register for state transition requests.
        (0x0014 => pub(crate) transition_ctrl: ReadWrite<u32, TRANSITION_CTRL::Register>),
        // 128bit token for conditional transitions.
        (0x0018 => pub(crate) transition_token: [ReadWrite<u32, TRANSITION_TOKEN::Register>; 4]),
        // This register exposes the decoded life cycle state.
        (0x0028 => pub(crate) transition_target: ReadWrite<u32, TRANSITION_TARGET::Register>),
        // Test/vendor-specific settings for the OTP macro wrapper.
        (0x002c => pub(crate) otp_vendor_test_ctrl: ReadWrite<u32, OTP_VENDOR_TEST_CTRL::Register>),
        // Test/vendor-specific settings for the OTP macro wrapper.
        (0x0030 => pub(crate) otp_vendor_test_status: ReadWrite<u32, OTP_VENDOR_TEST_STATUS::Register>),
        // This register exposes the decoded life cycle state.
        (0x0034 => pub(crate) lc_state: ReadWrite<u32, LC_STATE::Register>),
        // This register exposes the state of the decoded life cycle transition counter.
        (0x0038 => pub(crate) lc_transition_cnt: ReadWrite<u32, LC_TRANSITION_CNT::Register>),
        // This register exposes the id state of the device.
        (0x003c => pub(crate) lc_id_state: ReadWrite<u32, LC_ID_STATE::Register>),
        // This register holds the 32bit hardware revision,
        (0x0040 => pub(crate) hw_rev: ReadWrite<u32, HW_REV::Register>),
        // This is the 256bit DEVICE_ID value that is stored in the HW_CFG partition in OTP.
        (0x0044 => pub(crate) device_id: [ReadWrite<u32, DEVICE_ID::Register>; 8]),
        // This is a 256bit field used for keeping track of the manufacturing state.
        (0x0064 => pub(crate) manuf_state: [ReadWrite<u32, MANUF_STATE::Register>; 8]),
        (0x0084 => @END),
    }
}

register_bitfields![u32,
    pub(crate) ALERT_TEST [
        FATAL_PROG_ERROR OFFSET(0) NUMBITS(1) [],
        FATAL_STATE_ERROR OFFSET(1) NUMBITS(1) [],
        FATAL_BUS_INTEG_ERROR OFFSET(2) NUMBITS(1) [],
    ],
    pub(crate) STATUS [
        READY OFFSET(0) NUMBITS(1) [],
        TRANSITION_SUCCESSFUL OFFSET(1) NUMBITS(1) [],
        TRANSITION_COUNT_ERROR OFFSET(2) NUMBITS(1) [],
        TRANSITION_ERROR OFFSET(3) NUMBITS(1) [],
        TOKEN_ERROR OFFSET(4) NUMBITS(1) [],
        FLASH_RMA_ERROR OFFSET(5) NUMBITS(1) [],
        OTP_ERROR OFFSET(6) NUMBITS(1) [],
        STATE_ERROR OFFSET(7) NUMBITS(1) [],
        BUS_INTEG_ERROR OFFSET(8) NUMBITS(1) [],
        OTP_PARTITION_ERROR OFFSET(9) NUMBITS(1) [],
    ],
    pub(crate) CLAIM_TRANSITION_IF [
        MUTEX OFFSET(0) NUMBITS(8) [],
    ],
    pub(crate) TRANSITION_REGWEN [
        TRANSITION_REGWEN OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) TRANSITION_CMD [
        START OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) TRANSITION_CTRL [
        EXT_CLOCK_EN OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) TRANSITION_TOKEN [],
    pub(crate) TRANSITION_TARGET [
        STATE OFFSET(0) NUMBITS(30) [
            RAW = 0,
            TEST_UNLOCKED0 = 34636833,
            TEST_LOCKED0 = 69273666,
            TEST_UNLOCKED1 = 103910499,
            TEST_LOCKED1 = 138547332,
            TEST_UNLOCKED2 = 173184165,
            TEST_LOCKED2 = 207820998,
            TEST_UNLOCKED3 = 242457831,
            TEST_LOCKED3 = 277094664,
            TEST_UNLOCKED4 = 311731497,
            TEST_LOCKED4 = 346368330,
            TEST_UNLOCKED5 = 381005163,
            TEST_LOCKED5 = 415641996,
            TEST_UNLOCKED6 = 450278829,
            TEST_LOCKED6 = 484915662,
            TEST_UNLOCKED7 = 519552495,
            DEV = 554189328,
            PROD = 588826161,
            PROD_END = 623462994,
            RMA = 658099827,
            SCRAP = 692736660,
        ],
    ],
    pub(crate) OTP_VENDOR_TEST_CTRL [],
    pub(crate) OTP_VENDOR_TEST_STATUS [],
    pub(crate) LC_STATE [
        STATE OFFSET(0) NUMBITS(30) [
            RAW = 0,
            TEST_UNLOCKED0 = 34636833,
            TEST_LOCKED0 = 69273666,
            TEST_UNLOCKED1 = 103910499,
            TEST_LOCKED1 = 138547332,
            TEST_UNLOCKED2 = 173184165,
            TEST_LOCKED2 = 207820998,
            TEST_UNLOCKED3 = 242457831,
            TEST_LOCKED3 = 277094664,
            TEST_UNLOCKED4 = 311731497,
            TEST_LOCKED4 = 346368330,
            TEST_UNLOCKED5 = 381005163,
            TEST_LOCKED5 = 415641996,
            TEST_UNLOCKED6 = 450278829,
            TEST_LOCKED6 = 484915662,
            TEST_UNLOCKED7 = 519552495,
            DEV = 554189328,
            PROD = 588826161,
            PROD_END = 623462994,
            RMA = 658099827,
            SCRAP = 692736660,
            POST_TRANSITION = 727373493,
            ESCALATE = 762010326,
            INVALID = 796647159,
        ],
    ],
    pub(crate) LC_TRANSITION_CNT [
        CNT OFFSET(0) NUMBITS(5) [],
    ],
    pub(crate) LC_ID_STATE [],
    pub(crate) HW_REV [
        CHIP_REV OFFSET(0) NUMBITS(16) [],
        CHIP_GEN OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) DEVICE_ID [],
    pub(crate) MANUF_STATE [],
];

// End generated register constants for lc_ctrl
