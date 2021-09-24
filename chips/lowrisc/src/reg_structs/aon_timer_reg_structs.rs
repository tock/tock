// Generated register struct for aon_timer

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
    pub Aon_TimerRegisters {
        (0x0 => alert_test: WriteOnly<u32, ALERT_TEST::Register>),
        (0x4 => wkup_ctrl: ReadWrite<u32, WKUP_CTRL::Register>),
        (0x8 => wkup_thold: ReadWrite<u32, WKUP_THOLD::Register>),
        (0xc => wkup_count: ReadWrite<u32, WKUP_COUNT::Register>),
        (0x10 => wdog_regwen: ReadWrite<u32, WDOG_REGWEN::Register>),
        (0x14 => wdog_ctrl: ReadWrite<u32, WDOG_CTRL::Register>),
        (0x18 => wdog_bark_thold: ReadWrite<u32, WDOG_BARK_THOLD::Register>),
        (0x1c => wdog_bite_thold: ReadWrite<u32, WDOG_BITE_THOLD::Register>),
        (0x20 => wdog_count: ReadWrite<u32, WDOG_COUNT::Register>),
        (0x24 => intr_state: ReadWrite<u32, INTR_STATE::Register>),
        (0x28 => intr_test: WriteOnly<u32, INTR_TEST::Register>),
        (0x2c => wkup_cause: ReadWrite<u32, WKUP_CAUSE::Register>),
    }
}

register_bitfields![u32,
    ALERT_TEST [
        FATAL_FAULT OFFSET(0) NUMBITS(1) [],
    ],
    WKUP_CTRL [
        ENABLE OFFSET(0) NUMBITS(1) [],
        PRESCALER OFFSET(1) NUMBITS(12) [],
    ],
    WKUP_THOLD [
        THRESHOLD OFFSET(0) NUMBITS(32) [],
    ],
    WKUP_COUNT [
        COUNT OFFSET(0) NUMBITS(32) [],
    ],
    WDOG_REGWEN [
        REGWEN OFFSET(0) NUMBITS(1) [],
    ],
    WDOG_CTRL [
        ENABLE OFFSET(0) NUMBITS(1) [],
        PAUSE_IN_SLEEP OFFSET(1) NUMBITS(1) [],
    ],
    WDOG_BARK_THOLD [
        THRESHOLD OFFSET(0) NUMBITS(32) [],
    ],
    WDOG_BITE_THOLD [
        THRESHOLD OFFSET(0) NUMBITS(32) [],
    ],
    WDOG_COUNT [
        COUNT OFFSET(0) NUMBITS(32) [],
    ],
    INTR_STATE [
        WKUP_TIMER_EXPIRED OFFSET(0) NUMBITS(1) [],
        WDOG_TIMER_EXPIRED OFFSET(1) NUMBITS(1) [],
    ],
    INTR_TEST [
        WKUP_TIMER_EXPIRED OFFSET(0) NUMBITS(1) [],
        WDOG_TIMER_EXPIRED OFFSET(1) NUMBITS(1) [],
    ],
    WKUP_CAUSE [
        CAUSE OFFSET(0) NUMBITS(1) [],
    ],
];

// Number of alerts
pub const AON_TIMER_PARAM_NUM_ALERTS: u32 = 1;

// Register width
pub const AON_TIMER_PARAM_REG_WIDTH: u32 = 32;

// End generated register constants for aon_timer

