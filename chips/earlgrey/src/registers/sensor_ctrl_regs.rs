// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright lowRISC contributors 2023.

// Generated register constants for sensor_ctrl.
// Built for Earlgrey-M2.5.1-RC1-438-gacc67de99
// https://github.com/lowRISC/opentitan/tree/acc67de992ee8de5f2481b1b9580679850d8b5f5
// Tree status: clean
// Build date: 2023-08-08T00:15:38

// Original reference file: hw/top_earlgrey/ip/sensor_ctrl/data/sensor_ctrl.hjson
use kernel::utilities::registers::ReadWrite;
use kernel::utilities::registers::{register_bitfields, register_structs};
/// Number of alert events from ast
pub const SENSOR_CTRL_PARAM_NUM_ALERT_EVENTS: u32 = 11;
/// Number of local events
pub const SENSOR_CTRL_PARAM_NUM_LOCAL_EVENTS: u32 = 1;
/// Number of alerts sent from sensor control
pub const SENSOR_CTRL_PARAM_NUM_ALERTS: u32 = 2;
/// Number of IO rails
pub const SENSOR_CTRL_PARAM_NUM_IO_RAILS: u32 = 2;
/// Register width
pub const SENSOR_CTRL_PARAM_REG_WIDTH: u32 = 32;

register_structs! {
    pub SensorCtrlRegisters {
        /// Interrupt State Register
        (0x0000 => pub(crate) intr_state: ReadWrite<u32, INTR::Register>),
        /// Interrupt Enable Register
        (0x0004 => pub(crate) intr_enable: ReadWrite<u32, INTR::Register>),
        /// Interrupt Test Register
        (0x0008 => pub(crate) intr_test: ReadWrite<u32, INTR::Register>),
        /// Alert Test Register
        (0x000c => pub(crate) alert_test: ReadWrite<u32, ALERT_TEST::Register>),
        /// Controls the configurability of !!FATAL_ALERT_EN register.
        (0x0010 => pub(crate) cfg_regwen: ReadWrite<u32, CFG_REGWEN::Register>),
        /// Alert trigger test
        (0x0014 => pub(crate) alert_trig: [ReadWrite<u32, ALERT_TRIG::Register>; 1]),
        /// Each bit marks a corresponding alert as fatal or recoverable.
        (0x0018 => pub(crate) fatal_alert_en: [ReadWrite<u32, FATAL_ALERT_EN::Register>; 1]),
        /// Each bit represents a recoverable alert that has been triggered by AST.
        (0x001c => pub(crate) recov_alert: [ReadWrite<u32, RECOV_ALERT::Register>; 1]),
        /// Each bit represents a fatal alert that has been triggered by AST.
        (0x0020 => pub(crate) fatal_alert: [ReadWrite<u32, FATAL_ALERT::Register>; 1]),
        /// Status readback for ast
        (0x0024 => pub(crate) status: ReadWrite<u32, STATUS::Register>),
        (0x0028 => @END),
    }
}

register_bitfields![u32,
    /// Common Interrupt Offsets
    pub(crate) INTR [
        IO_STATUS_CHANGE OFFSET(0) NUMBITS(1) [],
        INIT_STATUS_CHANGE OFFSET(1) NUMBITS(1) [],
    ],
    pub(crate) ALERT_TEST [
        RECOV_ALERT OFFSET(0) NUMBITS(1) [],
        FATAL_ALERT OFFSET(1) NUMBITS(1) [],
    ],
    pub(crate) CFG_REGWEN [
        EN OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) ALERT_TRIG [
        VAL_0 OFFSET(0) NUMBITS(1) [],
        VAL_1 OFFSET(1) NUMBITS(1) [],
        VAL_2 OFFSET(2) NUMBITS(1) [],
        VAL_3 OFFSET(3) NUMBITS(1) [],
        VAL_4 OFFSET(4) NUMBITS(1) [],
        VAL_5 OFFSET(5) NUMBITS(1) [],
        VAL_6 OFFSET(6) NUMBITS(1) [],
        VAL_7 OFFSET(7) NUMBITS(1) [],
        VAL_8 OFFSET(8) NUMBITS(1) [],
        VAL_9 OFFSET(9) NUMBITS(1) [],
        VAL_10 OFFSET(10) NUMBITS(1) [],
    ],
    pub(crate) FATAL_ALERT_EN [
        VAL_0 OFFSET(0) NUMBITS(1) [],
        VAL_1 OFFSET(1) NUMBITS(1) [],
        VAL_2 OFFSET(2) NUMBITS(1) [],
        VAL_3 OFFSET(3) NUMBITS(1) [],
        VAL_4 OFFSET(4) NUMBITS(1) [],
        VAL_5 OFFSET(5) NUMBITS(1) [],
        VAL_6 OFFSET(6) NUMBITS(1) [],
        VAL_7 OFFSET(7) NUMBITS(1) [],
        VAL_8 OFFSET(8) NUMBITS(1) [],
        VAL_9 OFFSET(9) NUMBITS(1) [],
        VAL_10 OFFSET(10) NUMBITS(1) [],
    ],
    pub(crate) RECOV_ALERT [
        VAL_0 OFFSET(0) NUMBITS(1) [],
        VAL_1 OFFSET(1) NUMBITS(1) [],
        VAL_2 OFFSET(2) NUMBITS(1) [],
        VAL_3 OFFSET(3) NUMBITS(1) [],
        VAL_4 OFFSET(4) NUMBITS(1) [],
        VAL_5 OFFSET(5) NUMBITS(1) [],
        VAL_6 OFFSET(6) NUMBITS(1) [],
        VAL_7 OFFSET(7) NUMBITS(1) [],
        VAL_8 OFFSET(8) NUMBITS(1) [],
        VAL_9 OFFSET(9) NUMBITS(1) [],
        VAL_10 OFFSET(10) NUMBITS(1) [],
    ],
    pub(crate) FATAL_ALERT [
        VAL_0 OFFSET(0) NUMBITS(1) [],
        VAL_1 OFFSET(1) NUMBITS(1) [],
        VAL_2 OFFSET(2) NUMBITS(1) [],
        VAL_3 OFFSET(3) NUMBITS(1) [],
        VAL_4 OFFSET(4) NUMBITS(1) [],
        VAL_5 OFFSET(5) NUMBITS(1) [],
        VAL_6 OFFSET(6) NUMBITS(1) [],
        VAL_7 OFFSET(7) NUMBITS(1) [],
        VAL_8 OFFSET(8) NUMBITS(1) [],
        VAL_9 OFFSET(9) NUMBITS(1) [],
        VAL_10 OFFSET(10) NUMBITS(1) [],
        VAL_11 OFFSET(11) NUMBITS(1) [],
    ],
    pub(crate) STATUS [
        AST_INIT_DONE OFFSET(0) NUMBITS(1) [],
        IO_POK OFFSET(1) NUMBITS(2) [],
    ],
];

// End generated register constants for sensor_ctrl
