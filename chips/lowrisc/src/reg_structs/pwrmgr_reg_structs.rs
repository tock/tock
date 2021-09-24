// Generated register struct for PWRMGR

// Copyright information found in source file:
// Copyright lowRISC contributors.Copyright lowRISC contributors.

// Licensing information found in source file:
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

#[allow(unused_imports)]
use kernel::utilities::registers::{
    register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
};

register_structs! {
    pub PwrmgrRegisters {
        (0x0 => intr_state: ReadWrite<u32, INTR_STATE::Register>),
        (0x4 => intr_enable: ReadWrite<u32, INTR_ENABLE::Register>),
        (0x8 => intr_test: WriteOnly<u32, INTR_TEST::Register>),
        (0xc => ctrl_cfg_regwen: ReadOnly<u32, CTRL_CFG_REGWEN::Register>),
        (0x10 => control: ReadWrite<u32, CONTROL::Register>),
        (0x14 => cfg_cdc_sync: ReadWrite<u32, CFG_CDC_SYNC::Register>),
        (0x18 => wakeup_en_regwen: ReadWrite<u32, WAKEUP_EN_REGWEN::Register>),
        (0x1c => wakeup_en: ReadWrite<u32, WAKEUP_EN::Register>),
        (0x20 => wake_status: ReadOnly<u32, WAKE_STATUS::Register>),
        (0x24 => reset_en_regwen: ReadWrite<u32, RESET_EN_REGWEN::Register>),
        (0x28 => reset_en: ReadWrite<u32, RESET_EN::Register>),
        (0x2c => reset_status: ReadOnly<u32, RESET_STATUS::Register>),
        (0x30 => wake_info_capture_dis: ReadWrite<u32, WAKE_INFO_CAPTURE_DIS::Register>),
        (0x34 => wake_info: ReadWrite<u32, WAKE_INFO::Register>),
    }
}

register_bitfields![u32,
    INTR_STATE [
        WAKEUP OFFSET(0) NUMBITS(1) [],
    ],
    INTR_ENABLE [
        WAKEUP OFFSET(0) NUMBITS(1) [],
    ],
    INTR_TEST [
        WAKEUP OFFSET(0) NUMBITS(1) [],
    ],
    CTRL_CFG_REGWEN [
        EN OFFSET(0) NUMBITS(1) [],
    ],
    CONTROL [
        LOW_POWER_HINT OFFSET(0) NUMBITS(1) [
            NONE = 0,
            LOW_POWER = 1,
        ],
        CORE_CLK_EN OFFSET(4) NUMBITS(1) [
            DISABLED = 0,
            ENABLED = 1,
        ],
        IO_CLK_EN OFFSET(5) NUMBITS(1) [
            DISABLED = 0,
            ENABLED = 1,
        ],
        USB_CLK_EN_LP OFFSET(6) NUMBITS(1) [
            DISABLED = 0,
            ENABLED = 1,
        ],
        USB_CLK_EN_ACTIVE OFFSET(7) NUMBITS(1) [
            DISABLED = 0,
            ENABLED = 1,
        ],
        MAIN_PD_N OFFSET(8) NUMBITS(1) [
            POWER_DOWN = 0,
            POWER_UP = 1,
        ],
    ],
    CFG_CDC_SYNC [
        SYNC OFFSET(0) NUMBITS(1) [],
    ],
    WAKEUP_EN_REGWEN [
        EN OFFSET(0) NUMBITS(1) [],
    ],
    WAKEUP_EN [
        EN_0 OFFSET(0) NUMBITS(1) [],
    ],
    WAKE_STATUS [
        VAL_0 OFFSET(0) NUMBITS(1) [],
    ],
    RESET_EN_REGWEN [
        EN OFFSET(0) NUMBITS(1) [],
    ],
    RESET_EN [
        EN_0 OFFSET(0) NUMBITS(1) [],
    ],
    RESET_STATUS [
        VAL_0 OFFSET(0) NUMBITS(1) [],
    ],
    WAKE_INFO_CAPTURE_DIS [
        VAL OFFSET(0) NUMBITS(1) [],
    ],
    WAKE_INFO [
        REASONS OFFSET(0) NUMBITS(1) [],
        FALL_THROUGH OFFSET(1) NUMBITS(1) [],
        ABORT OFFSET(2) NUMBITS(1) [],
    ],
];

// Number of wakeups
pub const PWRMGR_PARAM_NUM_WKUPS: u32 = 1;

// Number of reset requets
pub const PWRMGR_PARAM_NUM_RST_REQS: u32 = 1;

// Register width
pub const PWRMGR_PARAM_REG_WIDTH: u32 = 32;

// Bit mask for enabled wakeups (common parameters)
pub const PWRMGR_WAKEUP_EN_EN_FIELD_WIDTH: u32 = 1;
pub const PWRMGR_WAKEUP_EN_EN_FIELDS_PER_REG: u32 = 32;
pub const PWRMGR_WAKEUP_EN_MULTIREG_COUNT: u32 = 1;

// A read only register of all current wake requests post enable mask (common
// parameters)
pub const PWRMGR_WAKE_STATUS_VAL_FIELD_WIDTH: u32 = 1;
pub const PWRMGR_WAKE_STATUS_VAL_FIELDS_PER_REG: u32 = 32;
pub const PWRMGR_WAKE_STATUS_MULTIREG_COUNT: u32 = 1;

// Bit mask for enabled reset requests (common parameters)
pub const PWRMGR_RESET_EN_EN_FIELD_WIDTH: u32 = 1;
pub const PWRMGR_RESET_EN_EN_FIELDS_PER_REG: u32 = 32;
pub const PWRMGR_RESET_EN_MULTIREG_COUNT: u32 = 1;

// A read only register of all current reset requests post enable mask
// (common parameters)
pub const PWRMGR_RESET_STATUS_VAL_FIELD_WIDTH: u32 = 1;
pub const PWRMGR_RESET_STATUS_VAL_FIELDS_PER_REG: u32 = 32;
pub const PWRMGR_RESET_STATUS_MULTIREG_COUNT: u32 = 1;

// End generated register constants for PWRMGR

