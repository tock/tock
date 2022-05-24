// Generated register constants for sram_ctrl.
// This file is licensed under either of:
//   Apache License, Version 2.0 (LICENSE-APACHE <http://www.apache.org/licenses/LICENSE-2.0>)
//   MIT License (LICENSE-MIT <http://opensource.org/licenses/MIT>)

// Built for earlgrey_silver_release_v5-5654-g222658011
// https://github.com/lowRISC/opentitan/tree/222658011c27d6c1f22f02c7f589043f207ff574
// Tree status: clean
// Build date: 2022-06-02T20:40:57

// Original reference file: hw/ip/sram_ctrl/data/sram_ctrl.hjson
// Copyright information found in the reference file:
//   Copyright lowRISC contributors.
// Licensing information found in the reference file:
//   Licensed under the Apache License, Version 2.0, see LICENSE for details.
//   SPDX-License-Identifier: Apache-2.0

use kernel::utilities::registers::ReadWrite;
use kernel::utilities::registers::{register_bitfields, register_structs};
// Number of alerts
pub const SRAM_CTRL_PARAM_NUM_ALERTS: u32 = 1;
// Register width
pub const SRAM_CTRL_PARAM_REG_WIDTH: u32 = 32;

register_structs! {
    pub SramCtrlRegisters {
        // Alert Test Register
        (0x0000 => pub(crate) alert_test: ReadWrite<u32, ALERT_TEST::Register>),
        // SRAM status register.
        (0x0004 => pub(crate) status: ReadWrite<u32, STATUS::Register>),
        // Lock register for execution enable register.
        (0x0008 => pub(crate) exec_regwen: ReadWrite<u32, EXEC_REGWEN::Register>),
        // Sram execution enable.
        (0x000c => pub(crate) exec: ReadWrite<u32, EXEC::Register>),
        // Lock register for control register.
        (0x0010 => pub(crate) ctrl_regwen: ReadWrite<u32, CTRL_REGWEN::Register>),
        // SRAM ctrl register.
        (0x0014 => pub(crate) ctrl: ReadWrite<u32, CTRL::Register>),
        (0x0018 => @END),
    }
}

register_bitfields![u32,
    pub(crate) ALERT_TEST [
        FATAL_ERROR OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) STATUS [
        BUS_INTEG_ERROR OFFSET(0) NUMBITS(1) [],
        INIT_ERROR OFFSET(1) NUMBITS(1) [],
        ESCALATED OFFSET(2) NUMBITS(1) [],
        SCR_KEY_VALID OFFSET(3) NUMBITS(1) [],
        SCR_KEY_SEED_VALID OFFSET(4) NUMBITS(1) [],
        INIT_DONE OFFSET(5) NUMBITS(1) [],
    ],
    pub(crate) EXEC_REGWEN [
        EXEC_REGWEN OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) EXEC [
        EN OFFSET(0) NUMBITS(4) [],
    ],
    pub(crate) CTRL_REGWEN [
        CTRL_REGWEN OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) CTRL [
        RENEW_SCR_KEY OFFSET(0) NUMBITS(1) [],
        INIT OFFSET(1) NUMBITS(1) [],
    ],
];

// End generated register constants for sram_ctrl
