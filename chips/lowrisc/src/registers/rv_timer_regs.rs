// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright lowRISC contributors 2023.

// Generated register constants for rv_timer.
// Built for Earlgrey-M2.5.1-RC1-438-gacc67de99
// https://github.com/lowRISC/opentitan/tree/acc67de992ee8de5f2481b1b9580679850d8b5f5
// Tree status: clean
// Build date: 2023-08-08T00:15:38

// Original reference file: hw/ip/rv_timer/data/rv_timer.hjson
use kernel::utilities::registers::ReadWrite;
use kernel::utilities::registers::{register_bitfields, register_structs};
/// Number of harts
pub const RV_TIMER_PARAM_N_HARTS: u32 = 1;
/// Number of timers per Hart
pub const RV_TIMER_PARAM_N_TIMERS: u32 = 1;
/// Number of alerts
pub const RV_TIMER_PARAM_NUM_ALERTS: u32 = 1;
/// Register width
pub const RV_TIMER_PARAM_REG_WIDTH: u32 = 32;

register_structs! {
    pub RvTimerRegisters {
        /// Alert Test Register
        (0x0000 => pub(crate) alert_test: ReadWrite<u32, ALERT_TEST::Register>),
        /// Control register
        (0x0004 => pub(crate) ctrl: [ReadWrite<u32, CTRL::Register>; 1]),
        (0x0008 => _reserved1),
        /// Interrupt Enable
        (0x0100 => pub(crate) intr_enable0: [ReadWrite<u32, INTR_ENABLE0::Register>; 1]),
        /// Interrupt Status
        (0x0104 => pub(crate) intr_state0: [ReadWrite<u32, INTR_STATE0::Register>; 1]),
        /// Interrupt test register
        (0x0108 => pub(crate) intr_test0: [ReadWrite<u32, INTR_TEST0::Register>; 1]),
        /// Configuration for Hart 0
        (0x010c => pub(crate) cfg0: ReadWrite<u32, CFG0::Register>),
        /// Timer value Lower
        (0x0110 => pub(crate) timer_v_lower0: ReadWrite<u32, TIMER_V_LOWER0::Register>),
        /// Timer value Upper
        (0x0114 => pub(crate) timer_v_upper0: ReadWrite<u32, TIMER_V_UPPER0::Register>),
        /// Timer value Lower
        (0x0118 => pub(crate) compare_lower0_0: ReadWrite<u32, COMPARE_LOWER0_0::Register>),
        /// Timer value Upper
        (0x011c => pub(crate) compare_upper0_0: ReadWrite<u32, COMPARE_UPPER0_0::Register>),
        (0x0120 => @END),
    }
}

register_bitfields![u32,
    pub(crate) ALERT_TEST [
        FATAL_FAULT OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) CTRL [
        ACTIVE_0 OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) INTR_ENABLE0 [
        IE_0 OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) INTR_STATE0 [
        IS_0 OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) INTR_TEST0 [
        T_0 OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) CFG0 [
        PRESCALE OFFSET(0) NUMBITS(12) [],
        STEP OFFSET(16) NUMBITS(8) [],
    ],
    pub(crate) TIMER_V_LOWER0 [
        V OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) TIMER_V_UPPER0 [
        V OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) COMPARE_LOWER0_0 [
        V OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) COMPARE_UPPER0_0 [
        V OFFSET(0) NUMBITS(32) [],
    ],
];

// End generated register constants for rv_timer
