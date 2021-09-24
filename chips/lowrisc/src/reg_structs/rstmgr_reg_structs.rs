// Generated register struct for RSTMGR

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
    pub RstmgrRegisters {
        (0x0 => reset_info: ReadWrite<u32, RESET_INFO::Register>),
        (0x4 => alert_info_ctrl: ReadWrite<u32, ALERT_INFO_CTRL::Register>),
        (0x8 => alert_info_attr: ReadOnly<u32, ALERT_INFO_ATTR::Register>),
        (0xc => alert_info: ReadOnly<u32, ALERT_INFO::Register>),
        (0x10 => sw_rst_regen: ReadWrite<u32, SW_RST_REGEN::Register>),
        (0x14 => sw_rst_ctrl_n: ReadWrite<u32, SW_RST_CTRL_N::Register>),
    }
}

register_bitfields![u32,
    RESET_INFO [
        POR OFFSET(0) NUMBITS(1) [],
        LOW_POWER_EXIT OFFSET(1) NUMBITS(1) [],
        NDM_RESET OFFSET(2) NUMBITS(1) [],
        HW_REQ OFFSET(3) NUMBITS(1) [],
    ],
    ALERT_INFO_CTRL [
        EN OFFSET(0) NUMBITS(1) [],
        INDEX OFFSET(4) NUMBITS(4) [],
    ],
    ALERT_INFO_ATTR [
        CNT_AVAIL OFFSET(0) NUMBITS(4) [],
    ],
    ALERT_INFO [
        VALUE OFFSET(0) NUMBITS(32) [],
    ],
    SW_RST_REGEN [
        EN_0 OFFSET(0) NUMBITS(1) [],
        EN_1 OFFSET(1) NUMBITS(1) [],
    ],
    SW_RST_CTRL_N [
        VAL_0 OFFSET(0) NUMBITS(1) [],
        VAL_1 OFFSET(1) NUMBITS(1) [],
    ],
];

// Read width for crash info
pub const RSTMGR_PARAM_RD_WIDTH: u32 = 32;

// Index width for crash info
pub const RSTMGR_PARAM_IDX_WIDTH: u32 = 4;

// Number of software resets
pub const RSTMGR_PARAM_NUM_SW_RESETS: u32 = 2;

// Register width
pub const RSTMGR_PARAM_REG_WIDTH: u32 = 32;

// Register write enable for software controllable resets.
pub const RSTMGR_SW_RST_REGEN_EN_FIELD_WIDTH: u32 = 1;
pub const RSTMGR_SW_RST_REGEN_EN_FIELDS_PER_REG: u32 = 32;
pub const RSTMGR_SW_RST_REGEN_MULTIREG_COUNT: u32 = 1;

// Software controllable resets.
pub const RSTMGR_SW_RST_CTRL_N_VAL_FIELD_WIDTH: u32 = 1;
pub const RSTMGR_SW_RST_CTRL_N_VAL_FIELDS_PER_REG: u32 = 32;
pub const RSTMGR_SW_RST_CTRL_N_MULTIREG_COUNT: u32 = 1;

// End generated register constants for RSTMGR

