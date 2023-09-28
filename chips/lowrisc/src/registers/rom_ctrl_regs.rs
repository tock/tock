// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright lowRISC contributors 2023.

// Generated register constants for rom_ctrl.
// Built for Earlgrey-M2.5.1-RC1-438-gacc67de99
// https://github.com/lowRISC/opentitan/tree/acc67de992ee8de5f2481b1b9580679850d8b5f5
// Tree status: clean
// Build date: 2023-08-08T00:15:38

// Original reference file: hw/ip/rom_ctrl/data/rom_ctrl.hjson
use kernel::utilities::registers::ReadWrite;
use kernel::utilities::registers::{register_bitfields, register_structs};
/// Number of alerts
pub const ROM_CTRL_PARAM_NUM_ALERTS: u32 = 1;
/// Register width
pub const ROM_CTRL_PARAM_REG_WIDTH: u32 = 32;

register_structs! {
    pub RomCtrlRegisters {
        /// Alert Test Register
        (0x0000 => pub(crate) alert_test: ReadWrite<u32, ALERT_TEST::Register>),
        /// The cause of a fatal alert.
        (0x0004 => pub(crate) fatal_alert_cause: ReadWrite<u32, FATAL_ALERT_CAUSE::Register>),
        /// The digest computed from the contents of ROM
        (0x0008 => pub(crate) digest: [ReadWrite<u32, DIGEST::Register>; 8]),
        /// The expected digest, stored in the top words of ROM
        (0x0028 => pub(crate) exp_digest: [ReadWrite<u32, EXP_DIGEST::Register>; 8]),
        (0x0048 => @END),
    }
}

register_bitfields![u32,
    pub(crate) ALERT_TEST [
        FATAL OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) FATAL_ALERT_CAUSE [
        CHECKER_ERROR OFFSET(0) NUMBITS(1) [],
        INTEGRITY_ERROR OFFSET(1) NUMBITS(1) [],
    ],
    pub(crate) DIGEST [
        DIGEST_0 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) EXP_DIGEST [
        DIGEST_0 OFFSET(0) NUMBITS(32) [],
    ],
];

// End generated register constants for rom_ctrl
