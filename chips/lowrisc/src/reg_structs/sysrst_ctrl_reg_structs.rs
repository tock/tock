// Generated register struct for sysrst_ctrl

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
    pub Sysrst_CtrlRegisters {
        (0x0 => intr_state: ReadWrite<u32, INTR_STATE::Register>),
        (0x4 => intr_enable: ReadWrite<u32, INTR_ENABLE::Register>),
        (0x8 => intr_test: WriteOnly<u32, INTR_TEST::Register>),
        (0xc => alert_test: WriteOnly<u32, ALERT_TEST::Register>),
        (0x10 => regwen: ReadWrite<u32, REGWEN::Register>),
        (0x14 => ec_rst_ctl: ReadWrite<u32, EC_RST_CTL::Register>),
        (0x18 => ulp_ac_debounce_ctl: ReadWrite<u32, ULP_AC_DEBOUNCE_CTL::Register>),
        (0x1c => ulp_lid_debounce_ctl: ReadWrite<u32, ULP_LID_DEBOUNCE_CTL::Register>),
        (0x20 => ulp_pwrb_debounce_ctl: ReadWrite<u32, ULP_PWRB_DEBOUNCE_CTL::Register>),
        (0x24 => ulp_ctl: ReadWrite<u32, ULP_CTL::Register>),
        (0x28 => ulp_status: ReadWrite<u32, ULP_STATUS::Register>),
        (0x2c => wkup_status: ReadWrite<u32, WKUP_STATUS::Register>),
        (0x30 => key_invert_ctl: ReadWrite<u32, KEY_INVERT_CTL::Register>),
        (0x34 => pin_allowed_ctl: ReadWrite<u32, PIN_ALLOWED_CTL::Register>),
        (0x38 => pin_out_ctl: ReadWrite<u32, PIN_OUT_CTL::Register>),
        (0x3c => pin_out_value: ReadWrite<u32, PIN_OUT_VALUE::Register>),
        (0x40 => pin_in_value: ReadOnly<u32, PIN_IN_VALUE::Register>),
        (0x44 => key_intr_ctl: ReadWrite<u32, KEY_INTR_CTL::Register>),
        (0x48 => key_intr_debounce_ctl: ReadWrite<u32, KEY_INTR_DEBOUNCE_CTL::Register>),
        (0x4c => auto_block_debounce_ctl: ReadWrite<u32, AUTO_BLOCK_DEBOUNCE_CTL::Register>),
        (0x50 => auto_block_out_ctl: ReadWrite<u32, AUTO_BLOCK_OUT_CTL::Register>),
        (0x54 => com_sel_ctl_0: ReadWrite<u32, COM_SEL_CTL_0::Register>),
        (0x58 => com_sel_ctl_1: ReadWrite<u32, COM_SEL_CTL_1::Register>),
        (0x5c => com_sel_ctl_2: ReadWrite<u32, COM_SEL_CTL_2::Register>),
        (0x60 => com_sel_ctl_3: ReadWrite<u32, COM_SEL_CTL_3::Register>),
        (0x64 => com_det_ctl_0: ReadWrite<u32, COM_DET_CTL_0::Register>),
        (0x68 => com_det_ctl_1: ReadWrite<u32, COM_DET_CTL_1::Register>),
        (0x6c => com_det_ctl_2: ReadWrite<u32, COM_DET_CTL_2::Register>),
        (0x70 => com_det_ctl_3: ReadWrite<u32, COM_DET_CTL_3::Register>),
        (0x74 => com_out_ctl_0: ReadWrite<u32, COM_OUT_CTL_0::Register>),
        (0x78 => com_out_ctl_1: ReadWrite<u32, COM_OUT_CTL_1::Register>),
        (0x7c => com_out_ctl_2: ReadWrite<u32, COM_OUT_CTL_2::Register>),
        (0x80 => com_out_ctl_3: ReadWrite<u32, COM_OUT_CTL_3::Register>),
        (0x84 => combo_intr_status: ReadWrite<u32, COMBO_INTR_STATUS::Register>),
        (0x88 => key_intr_status: ReadWrite<u32, KEY_INTR_STATUS::Register>),
    }
}

register_bitfields![u32,
    INTR_STATE [
        SYSRST_CTRL OFFSET(0) NUMBITS(1) [],
    ],
    INTR_ENABLE [
        SYSRST_CTRL OFFSET(0) NUMBITS(1) [],
    ],
    INTR_TEST [
        SYSRST_CTRL OFFSET(0) NUMBITS(1) [],
    ],
    ALERT_TEST [
        FATAL_FAULT OFFSET(0) NUMBITS(1) [],
    ],
    REGWEN [
        WRITE_EN OFFSET(0) NUMBITS(1) [],
    ],
    EC_RST_CTL [
        EC_RST_PULSE OFFSET(0) NUMBITS(16) [],
    ],
    ULP_AC_DEBOUNCE_CTL [
        ULP_AC_DEBOUNCE_TIMER OFFSET(0) NUMBITS(16) [],
    ],
    ULP_LID_DEBOUNCE_CTL [
        ULP_LID_DEBOUNCE_TIMER OFFSET(0) NUMBITS(16) [],
    ],
    ULP_PWRB_DEBOUNCE_CTL [
        ULP_PWRB_DEBOUNCE_TIMER OFFSET(0) NUMBITS(16) [],
    ],
    ULP_CTL [
        ULP_ENABLE OFFSET(0) NUMBITS(1) [],
    ],
    ULP_STATUS [
        ULP_WAKEUP OFFSET(0) NUMBITS(1) [],
    ],
    WKUP_STATUS [
        WAKEUP_STS OFFSET(0) NUMBITS(1) [],
    ],
    KEY_INVERT_CTL [
        KEY0_IN OFFSET(0) NUMBITS(1) [],
        KEY0_OUT OFFSET(1) NUMBITS(1) [],
        KEY1_IN OFFSET(2) NUMBITS(1) [],
        KEY1_OUT OFFSET(3) NUMBITS(1) [],
        KEY2_IN OFFSET(4) NUMBITS(1) [],
        KEY2_OUT OFFSET(5) NUMBITS(1) [],
        PWRB_IN OFFSET(6) NUMBITS(1) [],
        PWRB_OUT OFFSET(7) NUMBITS(1) [],
        AC_PRESENT OFFSET(8) NUMBITS(1) [],
        BAT_DISABLE OFFSET(9) NUMBITS(1) [],
        LID_OPEN OFFSET(10) NUMBITS(1) [],
        Z3_WAKEUP OFFSET(11) NUMBITS(1) [],
    ],
    PIN_ALLOWED_CTL [
        BAT_DISABLE_0 OFFSET(0) NUMBITS(1) [],
        EC_RST_L_0 OFFSET(1) NUMBITS(1) [],
        PWRB_OUT_0 OFFSET(2) NUMBITS(1) [],
        KEY0_OUT_0 OFFSET(3) NUMBITS(1) [],
        KEY1_OUT_0 OFFSET(4) NUMBITS(1) [],
        KEY2_OUT_0 OFFSET(5) NUMBITS(1) [],
        Z3_WAKEUP_0 OFFSET(6) NUMBITS(1) [],
        FLASH_WP_L_0 OFFSET(7) NUMBITS(1) [],
        BAT_DISABLE_1 OFFSET(8) NUMBITS(1) [],
        EC_RST_L_1 OFFSET(9) NUMBITS(1) [],
        PWRB_OUT_1 OFFSET(10) NUMBITS(1) [],
        KEY0_OUT_1 OFFSET(11) NUMBITS(1) [],
        KEY1_OUT_1 OFFSET(12) NUMBITS(1) [],
        KEY2_OUT_1 OFFSET(13) NUMBITS(1) [],
        Z3_WAKEUP_1 OFFSET(14) NUMBITS(1) [],
        FLASH_WP_L_1 OFFSET(15) NUMBITS(1) [],
    ],
    PIN_OUT_CTL [
        BAT_DISABLE OFFSET(0) NUMBITS(1) [],
        EC_RST_L OFFSET(1) NUMBITS(1) [],
        PWRB_OUT OFFSET(2) NUMBITS(1) [],
        KEY0_OUT OFFSET(3) NUMBITS(1) [],
        KEY1_OUT OFFSET(4) NUMBITS(1) [],
        KEY2_OUT OFFSET(5) NUMBITS(1) [],
        Z3_WAKEUP OFFSET(6) NUMBITS(1) [],
        FLASH_WP_L OFFSET(7) NUMBITS(1) [],
    ],
    PIN_OUT_VALUE [
        BAT_DISABLE OFFSET(0) NUMBITS(1) [],
        EC_RST_L OFFSET(1) NUMBITS(1) [],
        PWRB_OUT OFFSET(2) NUMBITS(1) [],
        KEY0_OUT OFFSET(3) NUMBITS(1) [],
        KEY1_OUT OFFSET(4) NUMBITS(1) [],
        KEY2_OUT OFFSET(5) NUMBITS(1) [],
        Z3_WAKEUP OFFSET(6) NUMBITS(1) [],
        FLASH_WP_L OFFSET(7) NUMBITS(1) [],
    ],
    PIN_IN_VALUE [
        AC_PRESENT OFFSET(0) NUMBITS(1) [],
        EC_RST_L OFFSET(1) NUMBITS(1) [],
        PWRB_IN OFFSET(2) NUMBITS(1) [],
        KEY0_IN OFFSET(3) NUMBITS(1) [],
        KEY1_IN OFFSET(4) NUMBITS(1) [],
        KEY2_IN OFFSET(5) NUMBITS(1) [],
        LID_OPEN OFFSET(6) NUMBITS(1) [],
    ],
    KEY_INTR_CTL [
        PWRB_IN_H2L OFFSET(0) NUMBITS(1) [],
        KEY0_IN_H2L OFFSET(1) NUMBITS(1) [],
        KEY1_IN_H2L OFFSET(2) NUMBITS(1) [],
        KEY2_IN_H2L OFFSET(3) NUMBITS(1) [],
        AC_PRESENT_H2L OFFSET(4) NUMBITS(1) [],
        EC_RST_L_H2L OFFSET(5) NUMBITS(1) [],
        PWRB_IN_L2H OFFSET(8) NUMBITS(1) [],
        KEY0_IN_L2H OFFSET(9) NUMBITS(1) [],
        KEY1_IN_L2H OFFSET(10) NUMBITS(1) [],
        KEY2_IN_L2H OFFSET(11) NUMBITS(1) [],
        AC_PRESENT_L2H OFFSET(12) NUMBITS(1) [],
        EC_RST_L_L2H OFFSET(13) NUMBITS(1) [],
    ],
    KEY_INTR_DEBOUNCE_CTL [
        DEBOUNCE_TIMER OFFSET(0) NUMBITS(16) [],
    ],
    AUTO_BLOCK_DEBOUNCE_CTL [
        DEBOUNCE_TIMER OFFSET(0) NUMBITS(16) [],
        AUTO_BLOCK_ENABLE OFFSET(16) NUMBITS(1) [],
    ],
    AUTO_BLOCK_OUT_CTL [
        KEY0_OUT_SEL OFFSET(0) NUMBITS(1) [],
        KEY1_OUT_SEL OFFSET(1) NUMBITS(1) [],
        KEY2_OUT_SEL OFFSET(2) NUMBITS(1) [],
        KEY0_OUT_VALUE OFFSET(4) NUMBITS(1) [],
        KEY1_OUT_VALUE OFFSET(5) NUMBITS(1) [],
        KEY2_OUT_VALUE OFFSET(6) NUMBITS(1) [],
    ],
    COM_SEL_CTL_0 [
        KEY0_IN_SEL_0 OFFSET(0) NUMBITS(1) [],
        KEY1_IN_SEL_0 OFFSET(1) NUMBITS(1) [],
        KEY2_IN_SEL_0 OFFSET(2) NUMBITS(1) [],
        PWRB_IN_SEL_0 OFFSET(3) NUMBITS(1) [],
        AC_PRESENT_SEL_0 OFFSET(4) NUMBITS(1) [],
    ],
    COM_SEL_CTL_1 [
        KEY0_IN_SEL_1 OFFSET(0) NUMBITS(1) [],
        KEY1_IN_SEL_1 OFFSET(1) NUMBITS(1) [],
        KEY2_IN_SEL_1 OFFSET(2) NUMBITS(1) [],
        PWRB_IN_SEL_1 OFFSET(3) NUMBITS(1) [],
        AC_PRESENT_SEL_1 OFFSET(4) NUMBITS(1) [],
    ],
    COM_SEL_CTL_2 [
        KEY0_IN_SEL_2 OFFSET(0) NUMBITS(1) [],
        KEY1_IN_SEL_2 OFFSET(1) NUMBITS(1) [],
        KEY2_IN_SEL_2 OFFSET(2) NUMBITS(1) [],
        PWRB_IN_SEL_2 OFFSET(3) NUMBITS(1) [],
        AC_PRESENT_SEL_2 OFFSET(4) NUMBITS(1) [],
    ],
    COM_SEL_CTL_3 [
        KEY0_IN_SEL_3 OFFSET(0) NUMBITS(1) [],
        KEY1_IN_SEL_3 OFFSET(1) NUMBITS(1) [],
        KEY2_IN_SEL_3 OFFSET(2) NUMBITS(1) [],
        PWRB_IN_SEL_3 OFFSET(3) NUMBITS(1) [],
        AC_PRESENT_SEL_3 OFFSET(4) NUMBITS(1) [],
    ],
    COM_DET_CTL_0 [
        DETECTION_TIMER_0 OFFSET(0) NUMBITS(32) [],
    ],
    COM_DET_CTL_1 [
        DETECTION_TIMER_1 OFFSET(0) NUMBITS(32) [],
    ],
    COM_DET_CTL_2 [
        DETECTION_TIMER_2 OFFSET(0) NUMBITS(32) [],
    ],
    COM_DET_CTL_3 [
        DETECTION_TIMER_3 OFFSET(0) NUMBITS(32) [],
    ],
    COM_OUT_CTL_0 [
        BAT_DISABLE_0 OFFSET(0) NUMBITS(1) [],
        INTERRUPT_0 OFFSET(1) NUMBITS(1) [],
        EC_RST_0 OFFSET(2) NUMBITS(1) [],
        RST_REQ_0 OFFSET(3) NUMBITS(1) [],
    ],
    COM_OUT_CTL_1 [
        BAT_DISABLE_1 OFFSET(0) NUMBITS(1) [],
        INTERRUPT_1 OFFSET(1) NUMBITS(1) [],
        EC_RST_1 OFFSET(2) NUMBITS(1) [],
        RST_REQ_1 OFFSET(3) NUMBITS(1) [],
    ],
    COM_OUT_CTL_2 [
        BAT_DISABLE_2 OFFSET(0) NUMBITS(1) [],
        INTERRUPT_2 OFFSET(1) NUMBITS(1) [],
        EC_RST_2 OFFSET(2) NUMBITS(1) [],
        RST_REQ_2 OFFSET(3) NUMBITS(1) [],
    ],
    COM_OUT_CTL_3 [
        BAT_DISABLE_3 OFFSET(0) NUMBITS(1) [],
        INTERRUPT_3 OFFSET(1) NUMBITS(1) [],
        EC_RST_3 OFFSET(2) NUMBITS(1) [],
        RST_REQ_3 OFFSET(3) NUMBITS(1) [],
    ],
    COMBO_INTR_STATUS [
        COMBO0_H2L OFFSET(0) NUMBITS(1) [],
        COMBO1_H2L OFFSET(1) NUMBITS(1) [],
        COMBO2_H2L OFFSET(2) NUMBITS(1) [],
        COMBO3_H2L OFFSET(3) NUMBITS(1) [],
    ],
    KEY_INTR_STATUS [
        PWRB_H2L OFFSET(0) NUMBITS(1) [],
        KEY0_IN_H2L OFFSET(1) NUMBITS(1) [],
        KEY1_IN_H2L OFFSET(2) NUMBITS(1) [],
        KEY2_IN_H2L OFFSET(3) NUMBITS(1) [],
        AC_PRESENT_H2L OFFSET(4) NUMBITS(1) [],
        EC_RST_L_H2L OFFSET(5) NUMBITS(1) [],
        PWRB_L2H OFFSET(6) NUMBITS(1) [],
        KEY0_IN_L2H OFFSET(7) NUMBITS(1) [],
        KEY1_IN_L2H OFFSET(8) NUMBITS(1) [],
        KEY2_IN_L2H OFFSET(9) NUMBITS(1) [],
        AC_PRESENT_L2H OFFSET(10) NUMBITS(1) [],
        EC_RST_L_L2H OFFSET(11) NUMBITS(1) [],
    ],
];

// Number of keyboard combos
pub const SYSRST_CTRL_PARAM_NUM_COMBO: u32 = 4;

// Number of timer bits
pub const SYSRST_CTRL_PARAM_TIMER_WIDTH: u32 = 16;

// Number of detection timer bits
pub const SYSRST_CTRL_PARAM_DET_TIMER_WIDTH: u32 = 32;

// Number of alerts
pub const SYSRST_CTRL_PARAM_NUM_ALERTS: u32 = 1;

// Register width
pub const SYSRST_CTRL_PARAM_REG_WIDTH: u32 = 32;

// To define the duration that the combo should be pressed
pub const SYSRST_CTRL_COM_DET_CTL_DETECTION_TIMER_FIELD_WIDTH: u32 = 32;
pub const SYSRST_CTRL_COM_DET_CTL_DETECTION_TIMER_FIELDS_PER_REG: u32 = 1;
pub const SYSRST_CTRL_COM_DET_CTL_MULTIREG_COUNT: u32 = 4;

// End generated register constants for sysrst_ctrl

