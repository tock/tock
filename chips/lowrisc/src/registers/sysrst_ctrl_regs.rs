// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright lowRISC contributors 2023.

// Generated register constants for sysrst_ctrl.
// Built for Earlgrey-M2.5.1-RC1-438-gacc67de99
// https://github.com/lowRISC/opentitan/tree/acc67de992ee8de5f2481b1b9580679850d8b5f5
// Tree status: clean
// Build date: 2023-08-08T00:15:38

// Original reference file: hw/ip/sysrst_ctrl/data/sysrst_ctrl.hjson
use kernel::utilities::registers::ReadWrite;
use kernel::utilities::registers::{register_bitfields, register_structs};
/// Number of keyboard combos
pub const SYSRST_CTRL_PARAM_NUM_COMBO: u32 = 4;
/// Number of timer bits
pub const SYSRST_CTRL_PARAM_TIMER_WIDTH: u32 = 16;
/// Number of detection timer bits
pub const SYSRST_CTRL_PARAM_DET_TIMER_WIDTH: u32 = 32;
/// Number of alerts
pub const SYSRST_CTRL_PARAM_NUM_ALERTS: u32 = 1;
/// Register width
pub const SYSRST_CTRL_PARAM_REG_WIDTH: u32 = 32;

register_structs! {
    pub SysrstCtrlRegisters {
        /// Interrupt State Register
        (0x0000 => pub(crate) intr_state: ReadWrite<u32, INTR::Register>),
        /// Interrupt Enable Register
        (0x0004 => pub(crate) intr_enable: ReadWrite<u32, INTR::Register>),
        /// Interrupt Test Register
        (0x0008 => pub(crate) intr_test: ReadWrite<u32, INTR::Register>),
        /// Alert Test Register
        (0x000c => pub(crate) alert_test: ReadWrite<u32, ALERT_TEST::Register>),
        /// Configuration write enable control register
        (0x0010 => pub(crate) regwen: ReadWrite<u32, REGWEN::Register>),
        /// EC reset control register
        (0x0014 => pub(crate) ec_rst_ctl: ReadWrite<u32, EC_RST_CTL::Register>),
        /// Ultra low power AC debounce control register
        (0x0018 => pub(crate) ulp_ac_debounce_ctl: ReadWrite<u32, ULP_AC_DEBOUNCE_CTL::Register>),
        /// Ultra low power lid debounce control register
        (0x001c => pub(crate) ulp_lid_debounce_ctl: ReadWrite<u32, ULP_LID_DEBOUNCE_CTL::Register>),
        /// Ultra low power pwrb debounce control register
        (0x0020 => pub(crate) ulp_pwrb_debounce_ctl: ReadWrite<u32, ULP_PWRB_DEBOUNCE_CTL::Register>),
        /// Ultra low power control register
        (0x0024 => pub(crate) ulp_ctl: ReadWrite<u32, ULP_CTL::Register>),
        /// Ultra low power status
        (0x0028 => pub(crate) ulp_status: ReadWrite<u32, ULP_STATUS::Register>),
        /// wakeup status
        (0x002c => pub(crate) wkup_status: ReadWrite<u32, WKUP_STATUS::Register>),
        /// configure key input output invert property
        (0x0030 => pub(crate) key_invert_ctl: ReadWrite<u32, KEY_INVERT_CTL::Register>),
        /// This register determines which override values are allowed for a given output.
        (0x0034 => pub(crate) pin_allowed_ctl: ReadWrite<u32, PIN_ALLOWED_CTL::Register>),
        /// Enables the override function for a specific pin.
        (0x0038 => pub(crate) pin_out_ctl: ReadWrite<u32, PIN_OUT_CTL::Register>),
        /// Sets the pin override value. Note that only the values
        (0x003c => pub(crate) pin_out_value: ReadWrite<u32, PIN_OUT_VALUE::Register>),
        /// For SW to read the sysrst_ctrl inputs like GPIO
        (0x0040 => pub(crate) pin_in_value: ReadWrite<u32, PIN_IN_VALUE::Register>),
        /// Define the keys or inputs that can trigger the interrupt
        (0x0044 => pub(crate) key_intr_ctl: ReadWrite<u32, KEY_INTR_CTL::Register>),
        /// Debounce timer control register for key-triggered interrupt
        (0x0048 => pub(crate) key_intr_debounce_ctl: ReadWrite<u32, KEY_INTR_DEBOUNCE_CTL::Register>),
        /// Debounce timer control register for pwrb_in H2L transition
        (0x004c => pub(crate) auto_block_debounce_ctl: ReadWrite<u32, AUTO_BLOCK_DEBOUNCE_CTL::Register>),
        /// configure the key outputs to auto-override and their value
        (0x0050 => pub(crate) auto_block_out_ctl: ReadWrite<u32, AUTO_BLOCK_OUT_CTL::Register>),
        /// To define the keys that define the pre-condition of the combo
        (0x0054 => pub(crate) com_pre_sel_ctl: [ReadWrite<u32, COM_PRE_SEL_CTL::Register>; 4]),
        /// To define the duration that the combo pre-condition should be pressed
        (0x0064 => pub(crate) com_pre_det_ctl: [ReadWrite<u32, COM_PRE_DET_CTL::Register>; 4]),
        /// To define the keys that trigger the combo
        (0x0074 => pub(crate) com_sel_ctl: [ReadWrite<u32, COM_SEL_CTL::Register>; 4]),
        /// To define the duration that the combo should be pressed
        (0x0084 => pub(crate) com_det_ctl: [ReadWrite<u32, COM_DET_CTL::Register>; 4]),
        /// To define the actions once the combo is detected
        (0x0094 => pub(crate) com_out_ctl: [ReadWrite<u32, COM_OUT_CTL::Register>; 4]),
        /// Combo interrupt source. These registers will only be set if the
        (0x00a4 => pub(crate) combo_intr_status: ReadWrite<u32, COMBO_INTR_STATUS::Register>),
        /// key interrupt source
        (0x00a8 => pub(crate) key_intr_status: ReadWrite<u32, KEY_INTR_STATUS::Register>),
        (0x00ac => @END),
    }
}

register_bitfields![u32,
    /// Common Interrupt Offsets
    pub(crate) INTR [
        EVENT_DETECTED OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) ALERT_TEST [
        FATAL_FAULT OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) REGWEN [
        WRITE_EN OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) EC_RST_CTL [
        EC_RST_PULSE OFFSET(0) NUMBITS(16) [],
    ],
    pub(crate) ULP_AC_DEBOUNCE_CTL [
        ULP_AC_DEBOUNCE_TIMER OFFSET(0) NUMBITS(16) [],
    ],
    pub(crate) ULP_LID_DEBOUNCE_CTL [
        ULP_LID_DEBOUNCE_TIMER OFFSET(0) NUMBITS(16) [],
    ],
    pub(crate) ULP_PWRB_DEBOUNCE_CTL [
        ULP_PWRB_DEBOUNCE_TIMER OFFSET(0) NUMBITS(16) [],
    ],
    pub(crate) ULP_CTL [
        ULP_ENABLE OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) ULP_STATUS [
        ULP_WAKEUP OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) WKUP_STATUS [
        WAKEUP_STS OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) KEY_INVERT_CTL [
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
    pub(crate) PIN_ALLOWED_CTL [
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
    pub(crate) PIN_OUT_CTL [
        BAT_DISABLE OFFSET(0) NUMBITS(1) [],
        EC_RST_L OFFSET(1) NUMBITS(1) [],
        PWRB_OUT OFFSET(2) NUMBITS(1) [],
        KEY0_OUT OFFSET(3) NUMBITS(1) [],
        KEY1_OUT OFFSET(4) NUMBITS(1) [],
        KEY2_OUT OFFSET(5) NUMBITS(1) [],
        Z3_WAKEUP OFFSET(6) NUMBITS(1) [],
        FLASH_WP_L OFFSET(7) NUMBITS(1) [],
    ],
    pub(crate) PIN_OUT_VALUE [
        BAT_DISABLE OFFSET(0) NUMBITS(1) [],
        EC_RST_L OFFSET(1) NUMBITS(1) [],
        PWRB_OUT OFFSET(2) NUMBITS(1) [],
        KEY0_OUT OFFSET(3) NUMBITS(1) [],
        KEY1_OUT OFFSET(4) NUMBITS(1) [],
        KEY2_OUT OFFSET(5) NUMBITS(1) [],
        Z3_WAKEUP OFFSET(6) NUMBITS(1) [],
        FLASH_WP_L OFFSET(7) NUMBITS(1) [],
    ],
    pub(crate) PIN_IN_VALUE [
        PWRB_IN OFFSET(0) NUMBITS(1) [],
        KEY0_IN OFFSET(1) NUMBITS(1) [],
        KEY1_IN OFFSET(2) NUMBITS(1) [],
        KEY2_IN OFFSET(3) NUMBITS(1) [],
        LID_OPEN OFFSET(4) NUMBITS(1) [],
        AC_PRESENT OFFSET(5) NUMBITS(1) [],
        EC_RST_L OFFSET(6) NUMBITS(1) [],
        FLASH_WP_L OFFSET(7) NUMBITS(1) [],
    ],
    pub(crate) KEY_INTR_CTL [
        PWRB_IN_H2L OFFSET(0) NUMBITS(1) [],
        KEY0_IN_H2L OFFSET(1) NUMBITS(1) [],
        KEY1_IN_H2L OFFSET(2) NUMBITS(1) [],
        KEY2_IN_H2L OFFSET(3) NUMBITS(1) [],
        AC_PRESENT_H2L OFFSET(4) NUMBITS(1) [],
        EC_RST_L_H2L OFFSET(5) NUMBITS(1) [],
        FLASH_WP_L_H2L OFFSET(6) NUMBITS(1) [],
        PWRB_IN_L2H OFFSET(7) NUMBITS(1) [],
        KEY0_IN_L2H OFFSET(8) NUMBITS(1) [],
        KEY1_IN_L2H OFFSET(9) NUMBITS(1) [],
        KEY2_IN_L2H OFFSET(10) NUMBITS(1) [],
        AC_PRESENT_L2H OFFSET(11) NUMBITS(1) [],
        EC_RST_L_L2H OFFSET(12) NUMBITS(1) [],
        FLASH_WP_L_L2H OFFSET(13) NUMBITS(1) [],
    ],
    pub(crate) KEY_INTR_DEBOUNCE_CTL [
        DEBOUNCE_TIMER OFFSET(0) NUMBITS(16) [],
    ],
    pub(crate) AUTO_BLOCK_DEBOUNCE_CTL [
        DEBOUNCE_TIMER OFFSET(0) NUMBITS(16) [],
        AUTO_BLOCK_ENABLE OFFSET(16) NUMBITS(1) [],
    ],
    pub(crate) AUTO_BLOCK_OUT_CTL [
        KEY0_OUT_SEL OFFSET(0) NUMBITS(1) [],
        KEY1_OUT_SEL OFFSET(1) NUMBITS(1) [],
        KEY2_OUT_SEL OFFSET(2) NUMBITS(1) [],
        KEY0_OUT_VALUE OFFSET(4) NUMBITS(1) [],
        KEY1_OUT_VALUE OFFSET(5) NUMBITS(1) [],
        KEY2_OUT_VALUE OFFSET(6) NUMBITS(1) [],
    ],
    pub(crate) COM_PRE_SEL_CTL [
        KEY0_IN_SEL_0 OFFSET(0) NUMBITS(1) [],
        KEY1_IN_SEL_0 OFFSET(1) NUMBITS(1) [],
        KEY2_IN_SEL_0 OFFSET(2) NUMBITS(1) [],
        PWRB_IN_SEL_0 OFFSET(3) NUMBITS(1) [],
        AC_PRESENT_SEL_0 OFFSET(4) NUMBITS(1) [],
    ],
    pub(crate) COM_PRE_DET_CTL [
        PRECONDITION_TIMER_0 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) COM_SEL_CTL [
        KEY0_IN_SEL_0 OFFSET(0) NUMBITS(1) [],
        KEY1_IN_SEL_0 OFFSET(1) NUMBITS(1) [],
        KEY2_IN_SEL_0 OFFSET(2) NUMBITS(1) [],
        PWRB_IN_SEL_0 OFFSET(3) NUMBITS(1) [],
        AC_PRESENT_SEL_0 OFFSET(4) NUMBITS(1) [],
    ],
    pub(crate) COM_DET_CTL [
        DETECTION_TIMER_0 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) COM_OUT_CTL [
        BAT_DISABLE_0 OFFSET(0) NUMBITS(1) [],
        INTERRUPT_0 OFFSET(1) NUMBITS(1) [],
        EC_RST_0 OFFSET(2) NUMBITS(1) [],
        RST_REQ_0 OFFSET(3) NUMBITS(1) [],
    ],
    pub(crate) COMBO_INTR_STATUS [
        COMBO0_H2L OFFSET(0) NUMBITS(1) [],
        COMBO1_H2L OFFSET(1) NUMBITS(1) [],
        COMBO2_H2L OFFSET(2) NUMBITS(1) [],
        COMBO3_H2L OFFSET(3) NUMBITS(1) [],
    ],
    pub(crate) KEY_INTR_STATUS [
        PWRB_H2L OFFSET(0) NUMBITS(1) [],
        KEY0_IN_H2L OFFSET(1) NUMBITS(1) [],
        KEY1_IN_H2L OFFSET(2) NUMBITS(1) [],
        KEY2_IN_H2L OFFSET(3) NUMBITS(1) [],
        AC_PRESENT_H2L OFFSET(4) NUMBITS(1) [],
        EC_RST_L_H2L OFFSET(5) NUMBITS(1) [],
        FLASH_WP_L_H2L OFFSET(6) NUMBITS(1) [],
        PWRB_L2H OFFSET(7) NUMBITS(1) [],
        KEY0_IN_L2H OFFSET(8) NUMBITS(1) [],
        KEY1_IN_L2H OFFSET(9) NUMBITS(1) [],
        KEY2_IN_L2H OFFSET(10) NUMBITS(1) [],
        AC_PRESENT_L2H OFFSET(11) NUMBITS(1) [],
        EC_RST_L_L2H OFFSET(12) NUMBITS(1) [],
        FLASH_WP_L_L2H OFFSET(13) NUMBITS(1) [],
    ],
];

// End generated register constants for sysrst_ctrl
