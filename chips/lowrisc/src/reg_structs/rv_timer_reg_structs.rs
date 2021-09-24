// Generated register struct for rv_timer

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
    pub Rv_TimerRegisters {
        (0x0 => alert_test: WriteOnly<u32, ALERT_TEST::Register>),
        (0x4 => ctrl: ReadWrite<u32, CTRL::Register>),
        (0x100 => cfg0: ReadWrite<u32, CFG0::Register>),
        (0x104 => timer_v_lower0: ReadWrite<u32, TIMER_V_LOWER0::Register>),
        (0x108 => timer_v_upper0: ReadWrite<u32, TIMER_V_UPPER0::Register>),
        (0x10c => compare_lower0_0: ReadWrite<u32, COMPARE_LOWER0_0::Register>),
        (0x110 => compare_upper0_0: ReadWrite<u32, COMPARE_UPPER0_0::Register>),
        (0x114 => intr_enable0: ReadWrite<u32, INTR_ENABLE0::Register>),
        (0x118 => intr_state0: ReadWrite<u32, INTR_STATE0::Register>),
        (0x11c => intr_test0: WriteOnly<u32, INTR_TEST0::Register>),
    }
}

register_bitfields![u32,
    ALERT_TEST [
        FATAL_FAULT OFFSET(0) NUMBITS(1) [],
    ],
    CTRL [
        ACTIVE_0 OFFSET(0) NUMBITS(1) [],
    ],
    CFG0 [
        PRESCALE OFFSET(0) NUMBITS(12) [],
        STEP OFFSET(16) NUMBITS(8) [],
    ],
    TIMER_V_LOWER0 [
        V OFFSET(0) NUMBITS(32) [],
    ],
    TIMER_V_UPPER0 [
        V OFFSET(0) NUMBITS(32) [],
    ],
    COMPARE_LOWER0_0 [
        V OFFSET(0) NUMBITS(32) [],
    ],
    COMPARE_UPPER0_0 [
        V OFFSET(0) NUMBITS(32) [],
    ],
    INTR_ENABLE0 [
        IE_0 OFFSET(0) NUMBITS(1) [],
    ],
    INTR_STATE0 [
        IS_0 OFFSET(0) NUMBITS(1) [],
    ],
    INTR_TEST0 [
        T_0 OFFSET(0) NUMBITS(1) [],
    ],
];

// Number of harts
pub const RV_TIMER_PARAM_N_HARTS: u32 = 1;

// Number of timers per Hart
pub const RV_TIMER_PARAM_N_TIMERS: u32 = 1;

// Number of alerts
pub const RV_TIMER_PARAM_NUM_ALERTS: u32 = 1;

// Register width
pub const RV_TIMER_PARAM_REG_WIDTH: u32 = 32;

// Control register (common parameters)
pub const RV_TIMER_CTRL_ACTIVE_FIELD_WIDTH: u32 = 1;
pub const RV_TIMER_CTRL_ACTIVE_FIELDS_PER_REG: u32 = 32;
pub const RV_TIMER_CTRL_MULTIREG_COUNT: u32 = 1;

// Interrupt Enable (common parameters)
pub const RV_TIMER_INTR_ENABLE0_IE_FIELD_WIDTH: u32 = 1;
pub const RV_TIMER_INTR_ENABLE0_IE_FIELDS_PER_REG: u32 = 32;
pub const RV_TIMER_INTR_ENABLE0_MULTIREG_COUNT: u32 = 1;

// Interrupt Status (common parameters)
pub const RV_TIMER_INTR_STATE0_IS_FIELD_WIDTH: u32 = 1;
pub const RV_TIMER_INTR_STATE0_IS_FIELDS_PER_REG: u32 = 32;
pub const RV_TIMER_INTR_STATE0_MULTIREG_COUNT: u32 = 1;

// Interrupt test register (common parameters)
pub const RV_TIMER_INTR_TEST0_T_FIELD_WIDTH: u32 = 1;
pub const RV_TIMER_INTR_TEST0_T_FIELDS_PER_REG: u32 = 32;
pub const RV_TIMER_INTR_TEST0_MULTIREG_COUNT: u32 = 1;

// End generated register constants for rv_timer

