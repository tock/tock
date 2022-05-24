// Generated register constants for aon_timer.
// This file is licensed under either of:
//   Apache License, Version 2.0 (LICENSE-APACHE <http://www.apache.org/licenses/LICENSE-2.0>)
//   MIT License (LICENSE-MIT <http://opensource.org/licenses/MIT>)

// Built for earlgrey_silver_release_v5-5654-g222658011
// https://github.com/lowRISC/opentitan/tree/222658011c27d6c1f22f02c7f589043f207ff574
// Tree status: clean
// Build date: 2022-06-02T20:40:57

// Original reference file: hw/ip/aon_timer/data/aon_timer.hjson
// Copyright information found in the reference file:
//   Copyright lowRISC contributors.
// Licensing information found in the reference file:
//   Licensed under the Apache License, Version 2.0, see LICENSE for details.
//   SPDX-License-Identifier: Apache-2.0

use kernel::utilities::registers::ReadWrite;
use kernel::utilities::registers::{register_bitfields, register_structs};
// Number of alerts
pub const AON_TIMER_PARAM_NUM_ALERTS: u32 = 1;
// Register width
pub const AON_TIMER_PARAM_REG_WIDTH: u32 = 32;

register_structs! {
    pub AonTimerRegisters {
        // Alert Test Register
        (0x0000 => pub(crate) alert_test: ReadWrite<u32, ALERT_TEST::Register>),
        // Wakeup Timer Control register
        (0x0004 => pub(crate) wkup_ctrl: ReadWrite<u32, WKUP_CTRL::Register>),
        // Wakeup Timer Threshold Register
        (0x0008 => pub(crate) wkup_thold: ReadWrite<u32, WKUP_THOLD::Register>),
        // Wakeup Timer Count Register
        (0x000c => pub(crate) wkup_count: ReadWrite<u32, WKUP_COUNT::Register>),
        // Watchdog Timer Write Enable Register
        (0x0010 => pub(crate) wdog_regwen: ReadWrite<u32, WDOG_REGWEN::Register>),
        // Watchdog Timer Control register
        (0x0014 => pub(crate) wdog_ctrl: ReadWrite<u32, WDOG_CTRL::Register>),
        // Watchdog Timer Bark Threshold Register
        (0x0018 => pub(crate) wdog_bark_thold: ReadWrite<u32, WDOG_BARK_THOLD::Register>),
        // Watchdog Timer Bite Threshold Register
        (0x001c => pub(crate) wdog_bite_thold: ReadWrite<u32, WDOG_BITE_THOLD::Register>),
        // Watchdog Timer Count Register
        (0x0020 => pub(crate) wdog_count: ReadWrite<u32, WDOG_COUNT::Register>),
        // Interrupt State Register
        (0x0024 => pub(crate) intr_state: ReadWrite<u32, INTR_STATE::Register>),
        // Interrupt Test Register
        (0x0028 => pub(crate) intr_test: ReadWrite<u32, INTR_TEST::Register>),
        // Wakeup request status
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
    pub(crate) WKUP_THOLD [],
    pub(crate) WKUP_COUNT [],
    pub(crate) WDOG_REGWEN [
        REGWEN OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) WDOG_CTRL [
        ENABLE OFFSET(0) NUMBITS(1) [],
        PAUSE_IN_SLEEP OFFSET(1) NUMBITS(1) [],
    ],
    pub(crate) WDOG_BARK_THOLD [],
    pub(crate) WDOG_BITE_THOLD [],
    pub(crate) WDOG_COUNT [],
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
