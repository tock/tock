// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright lowRISC contributors 2023.

// Generated register constants for aon_timer.
// Built for Earlgrey-M2.5.1-RC1-438-gacc67de99
// https://github.com/lowRISC/opentitan/tree/acc67de992ee8de5f2481b1b9580679850d8b5f5
// Tree status: clean
// Build date: 2023-08-08T00:15:38

// Original reference file: hw/ip/aon_timer/data/aon_timer.hjson
use kernel::utilities::registers::ReadWrite;
use kernel::utilities::registers::{register_bitfields, register_structs};
/// Number of alerts
pub const AON_TIMER_PARAM_NUM_ALERTS: u32 = 1;
/// Register width
pub const AON_TIMER_PARAM_REG_WIDTH: u32 = 32;

register_structs! {
    pub AonTimerRegisters {
        /// Alert Test Register
        (0x0000 => pub(crate) alert_test: ReadWrite<u32, ALERT_TEST::Register>),
        /// Wakeup Timer Control register
        (0x0004 => pub(crate) wkup_ctrl: ReadWrite<u32, WKUP_CTRL::Register>),
        /// Wakeup Timer Threshold Register
        (0x0008 => pub(crate) wkup_thold: ReadWrite<u32, WKUP_THOLD::Register>),
        /// Wakeup Timer Count Register
        (0x000c => pub(crate) wkup_count: ReadWrite<u32, WKUP_COUNT::Register>),
        /// Watchdog Timer Write Enable Register
        (0x0010 => pub(crate) wdog_regwen: ReadWrite<u32, WDOG_REGWEN::Register>),
        /// Watchdog Timer Control register
        (0x0014 => pub(crate) wdog_ctrl: ReadWrite<u32, WDOG_CTRL::Register>),
        /// Watchdog Timer Bark Threshold Register
        (0x0018 => pub(crate) wdog_bark_thold: ReadWrite<u32, WDOG_BARK_THOLD::Register>),
        /// Watchdog Timer Bite Threshold Register
        (0x001c => pub(crate) wdog_bite_thold: ReadWrite<u32, WDOG_BITE_THOLD::Register>),
        /// Watchdog Timer Count Register
        (0x0020 => pub(crate) wdog_count: ReadWrite<u32, WDOG_COUNT::Register>),
        /// Interrupt State Register
        (0x0024 => pub(crate) intr_state: ReadWrite<u32, INTR_STATE::Register>),
        /// Interrupt Test Register
        (0x0028 => pub(crate) intr_test: ReadWrite<u32, INTR_TEST::Register>),
        /// Wakeup request status
        (0x002c => pub(crate) wkup_cause: ReadWrite<u32, WKUP_CAUSE::Register>),
        (0x0030 => @END),
    }
}

register_bitfields![u32,
    pub(crate) ALERT_TEST [
        FATAL_FAULT OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) WKUP_CTRL [
        ENABLE OFFSET(0) NUMBITS(1) [],
        PRESCALER OFFSET(1) NUMBITS(12) [],
    ],
    pub(crate) WKUP_THOLD [
        THRESHOLD OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) WKUP_COUNT [
        COUNT OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) WDOG_REGWEN [
        REGWEN OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) WDOG_CTRL [
        ENABLE OFFSET(0) NUMBITS(1) [],
        PAUSE_IN_SLEEP OFFSET(1) NUMBITS(1) [],
    ],
    pub(crate) WDOG_BARK_THOLD [
        THRESHOLD OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) WDOG_BITE_THOLD [
        THRESHOLD OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) WDOG_COUNT [
        COUNT OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) INTR_STATE [
        WKUP_TIMER_EXPIRED OFFSET(0) NUMBITS(1) [],
        WDOG_TIMER_BARK OFFSET(1) NUMBITS(1) [],
    ],
    pub(crate) INTR_TEST [
        WKUP_TIMER_EXPIRED OFFSET(0) NUMBITS(1) [],
        WDOG_TIMER_BARK OFFSET(1) NUMBITS(1) [],
    ],
    pub(crate) WKUP_CAUSE [
        CAUSE OFFSET(0) NUMBITS(1) [],
    ],
];

// End generated register constants for aon_timer
