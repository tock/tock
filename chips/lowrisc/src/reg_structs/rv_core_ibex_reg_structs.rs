// Generated register struct for RV_CORE_IBEX

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
    pub Rv_Core_IbexRegisters {
        (0x0 => alert_test: WriteOnly<u32, ALERT_TEST::Register>),
        (0x4 => sw_alert_regwen_0: ReadWrite<u32, SW_ALERT_REGWEN_0::Register>),
        (0x8 => sw_alert_regwen_1: ReadWrite<u32, SW_ALERT_REGWEN_1::Register>),
        (0xc => sw_alert_0: ReadWrite<u32, SW_ALERT_0::Register>),
        (0x10 => sw_alert_1: ReadWrite<u32, SW_ALERT_1::Register>),
        (0x14 => ibus_regwen_0: ReadWrite<u32, IBUS_REGWEN_0::Register>),
        (0x18 => ibus_regwen_1: ReadWrite<u32, IBUS_REGWEN_1::Register>),
        (0x1c => ibus_addr_en_0: ReadWrite<u32, IBUS_ADDR_EN_0::Register>),
        (0x20 => ibus_addr_en_1: ReadWrite<u32, IBUS_ADDR_EN_1::Register>),
        (0x24 => ibus_addr_matching_0: ReadWrite<u32, IBUS_ADDR_MATCHING_0::Register>),
        (0x28 => ibus_addr_matching_1: ReadWrite<u32, IBUS_ADDR_MATCHING_1::Register>),
        (0x2c => ibus_remap_addr_0: ReadWrite<u32, IBUS_REMAP_ADDR_0::Register>),
        (0x30 => ibus_remap_addr_1: ReadWrite<u32, IBUS_REMAP_ADDR_1::Register>),
        (0x34 => dbus_regwen_0: ReadWrite<u32, DBUS_REGWEN_0::Register>),
        (0x38 => dbus_regwen_1: ReadWrite<u32, DBUS_REGWEN_1::Register>),
        (0x3c => dbus_addr_en_0: ReadWrite<u32, DBUS_ADDR_EN_0::Register>),
        (0x40 => dbus_addr_en_1: ReadWrite<u32, DBUS_ADDR_EN_1::Register>),
        (0x44 => dbus_addr_matching_0: ReadWrite<u32, DBUS_ADDR_MATCHING_0::Register>),
        (0x48 => dbus_addr_matching_1: ReadWrite<u32, DBUS_ADDR_MATCHING_1::Register>),
        (0x4c => dbus_remap_addr_0: ReadWrite<u32, DBUS_REMAP_ADDR_0::Register>),
        (0x50 => dbus_remap_addr_1: ReadWrite<u32, DBUS_REMAP_ADDR_1::Register>),
        (0x54 => nmi_enable: ReadWrite<u32, NMI_ENABLE::Register>),
        (0x58 => nmi_state: ReadWrite<u32, NMI_STATE::Register>),
        (0x5c => err_status: ReadWrite<u32, ERR_STATUS::Register>),
    }
}

register_bitfields![u32,
    ALERT_TEST [
        FATAL_SW_ERR OFFSET(0) NUMBITS(1) [],
        RECOV_SW_ERR OFFSET(1) NUMBITS(1) [],
        FATAL_HW_ERR OFFSET(2) NUMBITS(1) [],
        RECOV_HW_ERR OFFSET(3) NUMBITS(1) [],
    ],
    SW_ALERT_REGWEN_0 [
        EN_0 OFFSET(0) NUMBITS(1) [
            SOFTWARE_ALERT_LOCKED = 0,
            SOFTWARE_ALERT_ENABLED = 1,
        ],
    ],
    SW_ALERT_REGWEN_1 [
        EN_1 OFFSET(0) NUMBITS(1) [],
    ],
    SW_ALERT_0 [
        VAL_0 OFFSET(0) NUMBITS(2) [],
    ],
    SW_ALERT_1 [
        VAL_1 OFFSET(0) NUMBITS(2) [],
    ],
    IBUS_REGWEN_0 [
        EN_0 OFFSET(0) NUMBITS(1) [
            LOCKED = 0,
            ENABLED = 1,
        ],
    ],
    IBUS_REGWEN_1 [
        EN_1 OFFSET(0) NUMBITS(1) [],
    ],
    IBUS_ADDR_EN_0 [
        EN_0 OFFSET(0) NUMBITS(1) [],
    ],
    IBUS_ADDR_EN_1 [
        EN_1 OFFSET(0) NUMBITS(1) [],
    ],
    IBUS_ADDR_MATCHING_0 [
        VAL_0 OFFSET(0) NUMBITS(32) [],
    ],
    IBUS_ADDR_MATCHING_1 [
        VAL_1 OFFSET(0) NUMBITS(32) [],
    ],
    IBUS_REMAP_ADDR_0 [
        VAL_0 OFFSET(0) NUMBITS(32) [],
    ],
    IBUS_REMAP_ADDR_1 [
        VAL_1 OFFSET(0) NUMBITS(32) [],
    ],
    DBUS_REGWEN_0 [
        EN_0 OFFSET(0) NUMBITS(1) [
            LOCKED = 0,
            ENABLED = 1,
        ],
    ],
    DBUS_REGWEN_1 [
        EN_1 OFFSET(0) NUMBITS(1) [],
    ],
    DBUS_ADDR_EN_0 [
        EN_0 OFFSET(0) NUMBITS(1) [],
    ],
    DBUS_ADDR_EN_1 [
        EN_1 OFFSET(0) NUMBITS(1) [],
    ],
    DBUS_ADDR_MATCHING_0 [
        VAL_0 OFFSET(0) NUMBITS(32) [],
    ],
    DBUS_ADDR_MATCHING_1 [
        VAL_1 OFFSET(0) NUMBITS(32) [],
    ],
    DBUS_REMAP_ADDR_0 [
        VAL_0 OFFSET(0) NUMBITS(32) [],
    ],
    DBUS_REMAP_ADDR_1 [
        VAL_1 OFFSET(0) NUMBITS(32) [],
    ],
    NMI_ENABLE [
        ALERT_EN OFFSET(0) NUMBITS(1) [],
        WDOG_EN OFFSET(1) NUMBITS(1) [],
    ],
    NMI_STATE [
        ALERT OFFSET(0) NUMBITS(1) [],
        WDOG OFFSET(1) NUMBITS(1) [],
    ],
    ERR_STATUS [
        REG_INTG_ERR OFFSET(0) NUMBITS(1) [],
        FATAL_INTG_ERR OFFSET(8) NUMBITS(1) [],
        FATAL_CORE_ERR OFFSET(9) NUMBITS(1) [],
        RECOV_CORE_ERR OFFSET(10) NUMBITS(1) [],
    ],
];

// Number of software triggerable alerts
pub const RV_CORE_IBEX_PARAM_NUM_SW_ALERTS: u32 = 2;

// Number of translatable regions per ibex bus
pub const RV_CORE_IBEX_PARAM_NUM_REGIONS: u32 = 2;

// Number of alerts
pub const RV_CORE_IBEX_PARAM_NUM_ALERTS: u32 = 4;

// Register width
pub const RV_CORE_IBEX_PARAM_REG_WIDTH: u32 = 32;

// Software alert regwen. (common parameters)
pub const RV_CORE_IBEX_SW_ALERT_REGWEN_EN_FIELD_WIDTH: u32 = 1;
pub const RV_CORE_IBEX_SW_ALERT_REGWEN_EN_FIELDS_PER_REG: u32 = 32;
pub const RV_CORE_IBEX_SW_ALERT_REGWEN_MULTIREG_COUNT: u32 = 2;

//   Software trigger alerts.
pub const RV_CORE_IBEX_SW_ALERT_VAL_FIELD_WIDTH: u32 = 2;
pub const RV_CORE_IBEX_SW_ALERT_VAL_FIELDS_PER_REG: u32 = 16;
pub const RV_CORE_IBEX_SW_ALERT_MULTIREG_COUNT: u32 = 2;

// Ibus address control regwen. (common parameters)
pub const RV_CORE_IBEX_IBUS_REGWEN_EN_FIELD_WIDTH: u32 = 1;
pub const RV_CORE_IBEX_IBUS_REGWEN_EN_FIELDS_PER_REG: u32 = 32;
pub const RV_CORE_IBEX_IBUS_REGWEN_MULTIREG_COUNT: u32 = 2;

//   Enable Ibus address matching
pub const RV_CORE_IBEX_IBUS_ADDR_EN_EN_FIELD_WIDTH: u32 = 1;
pub const RV_CORE_IBEX_IBUS_ADDR_EN_EN_FIELDS_PER_REG: u32 = 32;
pub const RV_CORE_IBEX_IBUS_ADDR_EN_MULTIREG_COUNT: u32 = 2;

//   Matching region programming for ibus.
pub const RV_CORE_IBEX_IBUS_ADDR_MATCHING_VAL_FIELD_WIDTH: u32 = 32;
pub const RV_CORE_IBEX_IBUS_ADDR_MATCHING_VAL_FIELDS_PER_REG: u32 = 1;
pub const RV_CORE_IBEX_IBUS_ADDR_MATCHING_MULTIREG_COUNT: u32 = 2;

//   The remap address after a match has been made.
pub const RV_CORE_IBEX_IBUS_REMAP_ADDR_VAL_FIELD_WIDTH: u32 = 32;
pub const RV_CORE_IBEX_IBUS_REMAP_ADDR_VAL_FIELDS_PER_REG: u32 = 1;
pub const RV_CORE_IBEX_IBUS_REMAP_ADDR_MULTIREG_COUNT: u32 = 2;

// Dbus address control regwen. (common parameters)
pub const RV_CORE_IBEX_DBUS_REGWEN_EN_FIELD_WIDTH: u32 = 1;
pub const RV_CORE_IBEX_DBUS_REGWEN_EN_FIELDS_PER_REG: u32 = 32;
pub const RV_CORE_IBEX_DBUS_REGWEN_MULTIREG_COUNT: u32 = 2;

//   Enable dbus address matching
pub const RV_CORE_IBEX_DBUS_ADDR_EN_EN_FIELD_WIDTH: u32 = 1;
pub const RV_CORE_IBEX_DBUS_ADDR_EN_EN_FIELDS_PER_REG: u32 = 32;
pub const RV_CORE_IBEX_DBUS_ADDR_EN_MULTIREG_COUNT: u32 = 2;

//   See !!IBUS_ADDR_MATCHING for detailed description.
pub const RV_CORE_IBEX_DBUS_ADDR_MATCHING_VAL_FIELD_WIDTH: u32 = 32;
pub const RV_CORE_IBEX_DBUS_ADDR_MATCHING_VAL_FIELDS_PER_REG: u32 = 1;
pub const RV_CORE_IBEX_DBUS_ADDR_MATCHING_MULTIREG_COUNT: u32 = 2;

//   See !!IBUS_REMAP_ADDR for a detailed description.
pub const RV_CORE_IBEX_DBUS_REMAP_ADDR_VAL_FIELD_WIDTH: u32 = 32;
pub const RV_CORE_IBEX_DBUS_REMAP_ADDR_VAL_FIELDS_PER_REG: u32 = 1;
pub const RV_CORE_IBEX_DBUS_REMAP_ADDR_MULTIREG_COUNT: u32 = 2;

// End generated register constants for RV_CORE_IBEX

