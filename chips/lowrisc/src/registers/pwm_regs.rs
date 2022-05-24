// Generated register constants for pwm.
// This file is licensed under either of:
//   Apache License, Version 2.0 (LICENSE-APACHE <http://www.apache.org/licenses/LICENSE-2.0>)
//   MIT License (LICENSE-MIT <http://opensource.org/licenses/MIT>)

// Built for earlgrey_silver_release_v5-5654-g222658011
// https://github.com/lowRISC/opentitan/tree/222658011c27d6c1f22f02c7f589043f207ff574
// Tree status: clean
// Build date: 2022-06-02T20:40:57

// Original reference file: hw/ip/pwm/data/pwm.hjson
// Copyright information found in the reference file:
//   Copyright lowRISC contributors.
// Licensing information found in the reference file:
//   Licensed under the Apache License, Version 2.0, see LICENSE for details.
//   SPDX-License-Identifier: Apache-2.0

use kernel::utilities::registers::ReadWrite;
use kernel::utilities::registers::{register_bitfields, register_structs};
// Number of PWM outputs
pub const PWM_PARAM_N_OUTPUTS: u32 = 6;
// Number of alerts
pub const PWM_PARAM_NUM_ALERTS: u32 = 1;
// Register width
pub const PWM_PARAM_REG_WIDTH: u32 = 32;

register_structs! {
    pub PwmRegisters {
        // Alert Test Register
        (0x0000 => pub(crate) alert_test: ReadWrite<u32, ALERT_TEST::Register>),
        // Register write enable for all control registers
        (0x0004 => pub(crate) regwen: ReadWrite<u32, REGWEN::Register>),
        // Configuration register
        (0x0008 => pub(crate) cfg: ReadWrite<u32, CFG::Register>),
        // Enable PWM operation for each channel
        (0x000c => pub(crate) pwm_: [ReadWrite<u32, PWM_::Register>; 1]),
        // Invert the PWM output for each channel
        (0x0010 => pub(crate) inve: [ReadWrite<u32, INVE::Register>; 1]),
        // Basic PWM Channel Parameters
        (0x0014 => pub(crate) pwm_param: [ReadWrite<u32, PWM_PARAM::Register>; 6]),
        // Controls the duty_cycle of each channel.
        (0x002c => pub(crate) duty_cycle: [ReadWrite<u32, DUTY_CYCLE::Register>; 6]),
        // Hardware controlled blink/heartbeat parameters.
        (0x0044 => pub(crate) blink_param: [ReadWrite<u32, BLINK_PARAM::Register>; 6]),
        (0x005c => @END),
    }
}

register_bitfields![u32,
    pub(crate) ALERT_TEST [
        FATAL_FAULT OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) REGWEN [
        REGWEN OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) CFG [
        CLK_DIV OFFSET(0) NUMBITS(27) [],
        DC_RESN OFFSET(27) NUMBITS(4) [],
        CNTR_EN OFFSET(31) NUMBITS(1) [],
    ],
    pub(crate) PWM_ [
        EN_0 OFFSET(0) NUMBITS(1) [],
        EN_1 OFFSET(1) NUMBITS(1) [],
        EN_2 OFFSET(2) NUMBITS(1) [],
        EN_3 OFFSET(3) NUMBITS(1) [],
        EN_4 OFFSET(4) NUMBITS(1) [],
        EN_5 OFFSET(5) NUMBITS(1) [],
    ],
    pub(crate) INVE [
        INVERT_0 OFFSET(0) NUMBITS(1) [],
        INVERT_1 OFFSET(1) NUMBITS(1) [],
        INVERT_2 OFFSET(2) NUMBITS(1) [],
        INVERT_3 OFFSET(3) NUMBITS(1) [],
        INVERT_4 OFFSET(4) NUMBITS(1) [],
        INVERT_5 OFFSET(5) NUMBITS(1) [],
    ],
    pub(crate) PWM_PARAM [
        PHASE_DELAY_0 OFFSET(0) NUMBITS(16) [],
        HTBT_EN_0 OFFSET(30) NUMBITS(1) [],
        BLINK_EN_0 OFFSET(31) NUMBITS(1) [],
    ],
    pub(crate) DUTY_CYCLE [
        A_0 OFFSET(0) NUMBITS(16) [],
        B_0 OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) BLINK_PARAM [
        X_0 OFFSET(0) NUMBITS(16) [],
        Y_0 OFFSET(16) NUMBITS(16) [],
    ],
];

// End generated register constants for pwm
