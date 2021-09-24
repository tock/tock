// Generated register struct for pwm

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
    pub PwmRegisters {
        (0x0 => alert_test: WriteOnly<u32, ALERT_TEST::Register>),
        (0x4 => regen: ReadWrite<u32, REGEN::Register>),
        (0x8 => cfg: ReadWrite<u32, CFG::Register>),
        (0xc => pwm_en: ReadWrite<u32, PWM_EN::Register>),
        (0x10 => invert: ReadWrite<u32, INVERT::Register>),
        (0x14 => pwm_param_0: ReadWrite<u32, PWM_PARAM_0::Register>),
        (0x18 => pwm_param_1: ReadWrite<u32, PWM_PARAM_1::Register>),
        (0x1c => pwm_param_2: ReadWrite<u32, PWM_PARAM_2::Register>),
        (0x20 => pwm_param_3: ReadWrite<u32, PWM_PARAM_3::Register>),
        (0x24 => pwm_param_4: ReadWrite<u32, PWM_PARAM_4::Register>),
        (0x28 => pwm_param_5: ReadWrite<u32, PWM_PARAM_5::Register>),
        (0x2c => duty_cycle_0: ReadWrite<u32, DUTY_CYCLE_0::Register>),
        (0x30 => duty_cycle_1: ReadWrite<u32, DUTY_CYCLE_1::Register>),
        (0x34 => duty_cycle_2: ReadWrite<u32, DUTY_CYCLE_2::Register>),
        (0x38 => duty_cycle_3: ReadWrite<u32, DUTY_CYCLE_3::Register>),
        (0x3c => duty_cycle_4: ReadWrite<u32, DUTY_CYCLE_4::Register>),
        (0x40 => duty_cycle_5: ReadWrite<u32, DUTY_CYCLE_5::Register>),
        (0x44 => blink_param_0: ReadWrite<u32, BLINK_PARAM_0::Register>),
        (0x48 => blink_param_1: ReadWrite<u32, BLINK_PARAM_1::Register>),
        (0x4c => blink_param_2: ReadWrite<u32, BLINK_PARAM_2::Register>),
        (0x50 => blink_param_3: ReadWrite<u32, BLINK_PARAM_3::Register>),
        (0x54 => blink_param_4: ReadWrite<u32, BLINK_PARAM_4::Register>),
        (0x58 => blink_param_5: ReadWrite<u32, BLINK_PARAM_5::Register>),
    }
}

register_bitfields![u32,
    ALERT_TEST [
        FATAL_FAULT OFFSET(0) NUMBITS(1) [],
    ],
    REGEN [
        REGEN OFFSET(0) NUMBITS(1) [],
    ],
    CFG [
        CLK_DIV OFFSET(0) NUMBITS(27) [],
        DC_RESN OFFSET(27) NUMBITS(4) [],
        CNTR_EN OFFSET(31) NUMBITS(1) [],
    ],
    PWM_EN [
        EN_0 OFFSET(0) NUMBITS(1) [],
        EN_1 OFFSET(1) NUMBITS(1) [],
        EN_2 OFFSET(2) NUMBITS(1) [],
        EN_3 OFFSET(3) NUMBITS(1) [],
        EN_4 OFFSET(4) NUMBITS(1) [],
        EN_5 OFFSET(5) NUMBITS(1) [],
    ],
    INVERT [
        INVERT_0 OFFSET(0) NUMBITS(1) [],
        INVERT_1 OFFSET(1) NUMBITS(1) [],
        INVERT_2 OFFSET(2) NUMBITS(1) [],
        INVERT_3 OFFSET(3) NUMBITS(1) [],
        INVERT_4 OFFSET(4) NUMBITS(1) [],
        INVERT_5 OFFSET(5) NUMBITS(1) [],
    ],
    PWM_PARAM_0 [
        PHASE_DELAY_0 OFFSET(0) NUMBITS(16) [],
        HTBT_EN_0 OFFSET(30) NUMBITS(1) [],
        BLINK_EN_0 OFFSET(31) NUMBITS(1) [],
    ],
    PWM_PARAM_1 [
        PHASE_DELAY_1 OFFSET(0) NUMBITS(16) [],
        HTBT_EN_1 OFFSET(30) NUMBITS(1) [],
        BLINK_EN_1 OFFSET(31) NUMBITS(1) [],
    ],
    PWM_PARAM_2 [
        PHASE_DELAY_2 OFFSET(0) NUMBITS(16) [],
        HTBT_EN_2 OFFSET(30) NUMBITS(1) [],
        BLINK_EN_2 OFFSET(31) NUMBITS(1) [],
    ],
    PWM_PARAM_3 [
        PHASE_DELAY_3 OFFSET(0) NUMBITS(16) [],
        HTBT_EN_3 OFFSET(30) NUMBITS(1) [],
        BLINK_EN_3 OFFSET(31) NUMBITS(1) [],
    ],
    PWM_PARAM_4 [
        PHASE_DELAY_4 OFFSET(0) NUMBITS(16) [],
        HTBT_EN_4 OFFSET(30) NUMBITS(1) [],
        BLINK_EN_4 OFFSET(31) NUMBITS(1) [],
    ],
    PWM_PARAM_5 [
        PHASE_DELAY_5 OFFSET(0) NUMBITS(16) [],
        HTBT_EN_5 OFFSET(30) NUMBITS(1) [],
        BLINK_EN_5 OFFSET(31) NUMBITS(1) [],
    ],
    DUTY_CYCLE_0 [
        A_0 OFFSET(0) NUMBITS(16) [],
        B_0 OFFSET(16) NUMBITS(16) [],
    ],
    DUTY_CYCLE_1 [
        A_1 OFFSET(0) NUMBITS(16) [],
        B_1 OFFSET(16) NUMBITS(16) [],
    ],
    DUTY_CYCLE_2 [
        A_2 OFFSET(0) NUMBITS(16) [],
        B_2 OFFSET(16) NUMBITS(16) [],
    ],
    DUTY_CYCLE_3 [
        A_3 OFFSET(0) NUMBITS(16) [],
        B_3 OFFSET(16) NUMBITS(16) [],
    ],
    DUTY_CYCLE_4 [
        A_4 OFFSET(0) NUMBITS(16) [],
        B_4 OFFSET(16) NUMBITS(16) [],
    ],
    DUTY_CYCLE_5 [
        A_5 OFFSET(0) NUMBITS(16) [],
        B_5 OFFSET(16) NUMBITS(16) [],
    ],
    BLINK_PARAM_0 [
        X_0 OFFSET(0) NUMBITS(16) [],
        Y_0 OFFSET(16) NUMBITS(16) [],
    ],
    BLINK_PARAM_1 [
        X_1 OFFSET(0) NUMBITS(16) [],
        Y_1 OFFSET(16) NUMBITS(16) [],
    ],
    BLINK_PARAM_2 [
        X_2 OFFSET(0) NUMBITS(16) [],
        Y_2 OFFSET(16) NUMBITS(16) [],
    ],
    BLINK_PARAM_3 [
        X_3 OFFSET(0) NUMBITS(16) [],
        Y_3 OFFSET(16) NUMBITS(16) [],
    ],
    BLINK_PARAM_4 [
        X_4 OFFSET(0) NUMBITS(16) [],
        Y_4 OFFSET(16) NUMBITS(16) [],
    ],
    BLINK_PARAM_5 [
        X_5 OFFSET(0) NUMBITS(16) [],
        Y_5 OFFSET(16) NUMBITS(16) [],
    ],
];

// Number of PWM outputs
pub const PWM_PARAM_N_OUTPUTS: u32 = 6;

// Number of alerts
pub const PWM_PARAM_NUM_ALERTS: u32 = 1;

// Register width
pub const PWM_PARAM_REG_WIDTH: u32 = 32;

// Enable PWM operation for each channel (common parameters)
pub const PWM_PWM_EN_EN_FIELD_WIDTH: u32 = 1;
pub const PWM_PWM_EN_EN_FIELDS_PER_REG: u32 = 32;
pub const PWM_PWM_EN_MULTIREG_COUNT: u32 = 1;

// Invert the PWM output for each channel (common parameters)
pub const PWM_INVERT_INVERT_FIELD_WIDTH: u32 = 1;
pub const PWM_INVERT_INVERT_FIELDS_PER_REG: u32 = 32;
pub const PWM_INVERT_MULTIREG_COUNT: u32 = 1;

// End generated register constants for pwm

