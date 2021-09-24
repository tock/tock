// Generated register struct for sram_ctrl

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
    pub Sram_CtrlRegisters {
        (0x0 => alert_test: WriteOnly<u32, ALERT_TEST::Register>),
        (0x4 => status: ReadOnly<u32, STATUS::Register>),
        (0x8 => exec_regwen: ReadWrite<u32, EXEC_REGWEN::Register>),
        (0xc => exec: ReadWrite<u32, EXEC::Register>),
        (0x10 => ctrl_regwen: ReadWrite<u32, CTRL_REGWEN::Register>),
        (0x14 => ctrl: WriteOnly<u32, CTRL::Register>),
    }
}

register_bitfields![u32,
    ALERT_TEST [
        FATAL_ERROR OFFSET(0) NUMBITS(1) [],
    ],
    STATUS [
        BUS_INTEG_ERROR OFFSET(0) NUMBITS(1) [],
        INIT_ERROR OFFSET(1) NUMBITS(1) [],
        ESCALATED OFFSET(2) NUMBITS(1) [],
        SCR_KEY_VALID OFFSET(3) NUMBITS(1) [],
        SCR_KEY_SEED_VALID OFFSET(4) NUMBITS(1) [],
        INIT_DONE OFFSET(5) NUMBITS(1) [],
    ],
    EXEC_REGWEN [
        EXEC_REGWEN OFFSET(0) NUMBITS(1) [],
    ],
    EXEC [
        EN OFFSET(0) NUMBITS(3) [],
    ],
    CTRL_REGWEN [
        CTRL_REGWEN OFFSET(0) NUMBITS(1) [],
    ],
    CTRL [
        RENEW_SCR_KEY OFFSET(0) NUMBITS(1) [],
        INIT OFFSET(1) NUMBITS(1) [],
    ],
];

// Number of alerts
pub const SRAM_CTRL_PARAM_NUM_ALERTS: u32 = 1;

// Register width
pub const SRAM_CTRL_PARAM_REG_WIDTH: u32 = 32;

// End generated register constants for sram_ctrl

