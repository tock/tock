// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright lowRISC contributors 2023.

// Generated register constants for rstmgr.
// Built for Earlgrey-M2.5.1-RC1-438-gacc67de99
// https://github.com/lowRISC/opentitan/tree/acc67de992ee8de5f2481b1b9580679850d8b5f5
// Tree status: clean
// Build date: 2023-08-08T00:15:38

// Original reference file: hw/top_earlgrey/ip/rstmgr/data/autogen/rstmgr.hjson
use kernel::utilities::registers::ReadWrite;
use kernel::utilities::registers::{register_bitfields, register_structs};
/// Read width for crash info
pub const RSTMGR_PARAM_RD_WIDTH: u32 = 32;
/// Index width for crash info
pub const RSTMGR_PARAM_IDX_WIDTH: u32 = 4;
/// Number of hardware reset requests, inclusive of debug resets and pwrmgr's internal resets
pub const RSTMGR_PARAM_NUM_HW_RESETS: u32 = 5;
/// Number of software resets
pub const RSTMGR_PARAM_NUM_SW_RESETS: u32 = 8;
/// Number of total reset requests, inclusive of hw/sw, por and low power exit
pub const RSTMGR_PARAM_NUM_TOTAL_RESETS: u32 = 8;
/// Number of alerts
pub const RSTMGR_PARAM_NUM_ALERTS: u32 = 2;
/// Register width
pub const RSTMGR_PARAM_REG_WIDTH: u32 = 32;

register_structs! {
    pub RstmgrRegisters {
        /// Alert Test Register
        (0x0000 => pub(crate) alert_test: ReadWrite<u32, ALERT_TEST::Register>),
        /// Software requested system reset.
        (0x0004 => pub(crate) reset_req: ReadWrite<u32, RESET_REQ::Register>),
        /// Device reset reason.
        (0x0008 => pub(crate) reset_info: ReadWrite<u32, RESET_INFO::Register>),
        /// Alert write enable
        (0x000c => pub(crate) alert_regwen: ReadWrite<u32, ALERT_REGWEN::Register>),
        /// Alert info dump controls.
        (0x0010 => pub(crate) alert_info_ctrl: ReadWrite<u32, ALERT_INFO_CTRL::Register>),
        /// Alert info dump attributes.
        (0x0014 => pub(crate) alert_info_attr: ReadWrite<u32, ALERT_INFO_ATTR::Register>),
        ///   Alert dump information prior to last reset.
        (0x0018 => pub(crate) alert_info: ReadWrite<u32, ALERT_INFO::Register>),
        /// Cpu write enable
        (0x001c => pub(crate) cpu_regwen: ReadWrite<u32, CPU_REGWEN::Register>),
        /// Cpu info dump controls.
        (0x0020 => pub(crate) cpu_info_ctrl: ReadWrite<u32, CPU_INFO_CTRL::Register>),
        /// Cpu info dump attributes.
        (0x0024 => pub(crate) cpu_info_attr: ReadWrite<u32, CPU_INFO_ATTR::Register>),
        ///   Cpu dump information prior to last reset.
        (0x0028 => pub(crate) cpu_info: ReadWrite<u32, CPU_INFO::Register>),
        /// Register write enable for software controllable resets.
        (0x002c => pub(crate) sw_rst_regwen: [ReadWrite<u32, SW_RST_REGWEN::Register>; 8]),
        /// Software controllable resets.
        (0x004c => pub(crate) sw_rst_ctrl_n: [ReadWrite<u32, SW_RST_CTRL_N::Register>; 8]),
        /// A bit vector of all the errors that have occurred in reset manager
        (0x006c => pub(crate) err_code: ReadWrite<u32, ERR_CODE::Register>),
        (0x0070 => @END),
    }
}

register_bitfields![u32,
    pub(crate) ALERT_TEST [
        FATAL_FAULT OFFSET(0) NUMBITS(1) [],
        FATAL_CNSTY_FAULT OFFSET(1) NUMBITS(1) [],
    ],
    pub(crate) RESET_REQ [
        VAL OFFSET(0) NUMBITS(4) [],
    ],
    pub(crate) RESET_INFO [
        POR OFFSET(0) NUMBITS(1) [],
        LOW_POWER_EXIT OFFSET(1) NUMBITS(1) [],
        SW_RESET OFFSET(2) NUMBITS(1) [],
        HW_REQ OFFSET(3) NUMBITS(5) [],
    ],
    pub(crate) ALERT_REGWEN [
        EN OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) ALERT_INFO_CTRL [
        EN OFFSET(0) NUMBITS(1) [],
        INDEX OFFSET(4) NUMBITS(4) [],
    ],
    pub(crate) ALERT_INFO_ATTR [
        CNT_AVAIL OFFSET(0) NUMBITS(4) [],
    ],
    pub(crate) ALERT_INFO [
        VALUE OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) CPU_REGWEN [
        EN OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) CPU_INFO_CTRL [
        EN OFFSET(0) NUMBITS(1) [],
        INDEX OFFSET(4) NUMBITS(4) [],
    ],
    pub(crate) CPU_INFO_ATTR [
        CNT_AVAIL OFFSET(0) NUMBITS(4) [],
    ],
    pub(crate) CPU_INFO [
        VALUE OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) SW_RST_REGWEN [
        EN_0 OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) SW_RST_CTRL_N [
        VAL_0 OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) ERR_CODE [
        REG_INTG_ERR OFFSET(0) NUMBITS(1) [],
        RESET_CONSISTENCY_ERR OFFSET(1) NUMBITS(1) [],
        FSM_ERR OFFSET(2) NUMBITS(1) [],
    ],
];

// End generated register constants for rstmgr
