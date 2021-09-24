// Generated register struct for adc_ctrl

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
    pub Adc_CtrlRegisters {
        (0x0 => intr_state: ReadWrite<u32, INTR_STATE::Register>),
        (0x4 => intr_enable: ReadWrite<u32, INTR_ENABLE::Register>),
        (0x8 => intr_test: WriteOnly<u32, INTR_TEST::Register>),
        (0xc => alert_test: WriteOnly<u32, ALERT_TEST::Register>),
        (0x10 => adc_en_ctl: ReadWrite<u32, adc_en_ctl::Register>),
        (0x14 => adc_pd_ctl: ReadWrite<u32, adc_pd_ctl::Register>),
        (0x18 => adc_lp_sample_ctl: ReadWrite<u32, adc_lp_sample_ctl::Register>),
        (0x1c => adc_sample_ctl: ReadWrite<u32, adc_sample_ctl::Register>),
        (0x20 => adc_fsm_rst: ReadWrite<u32, adc_fsm_rst::Register>),
        (0x24 => adc_chn0_filter_ctl_0: ReadWrite<u32, adc_chn0_filter_ctl_0::Register>),
        (0x28 => adc_chn0_filter_ctl_1: ReadWrite<u32, adc_chn0_filter_ctl_1::Register>),
        (0x2c => adc_chn0_filter_ctl_2: ReadWrite<u32, adc_chn0_filter_ctl_2::Register>),
        (0x30 => adc_chn0_filter_ctl_3: ReadWrite<u32, adc_chn0_filter_ctl_3::Register>),
        (0x34 => adc_chn0_filter_ctl_4: ReadWrite<u32, adc_chn0_filter_ctl_4::Register>),
        (0x38 => adc_chn0_filter_ctl_5: ReadWrite<u32, adc_chn0_filter_ctl_5::Register>),
        (0x3c => adc_chn0_filter_ctl_6: ReadWrite<u32, adc_chn0_filter_ctl_6::Register>),
        (0x40 => adc_chn0_filter_ctl_7: ReadWrite<u32, adc_chn0_filter_ctl_7::Register>),
        (0x44 => adc_chn1_filter_ctl_0: ReadWrite<u32, adc_chn1_filter_ctl_0::Register>),
        (0x48 => adc_chn1_filter_ctl_1: ReadWrite<u32, adc_chn1_filter_ctl_1::Register>),
        (0x4c => adc_chn1_filter_ctl_2: ReadWrite<u32, adc_chn1_filter_ctl_2::Register>),
        (0x50 => adc_chn1_filter_ctl_3: ReadWrite<u32, adc_chn1_filter_ctl_3::Register>),
        (0x54 => adc_chn1_filter_ctl_4: ReadWrite<u32, adc_chn1_filter_ctl_4::Register>),
        (0x58 => adc_chn1_filter_ctl_5: ReadWrite<u32, adc_chn1_filter_ctl_5::Register>),
        (0x5c => adc_chn1_filter_ctl_6: ReadWrite<u32, adc_chn1_filter_ctl_6::Register>),
        (0x60 => adc_chn1_filter_ctl_7: ReadWrite<u32, adc_chn1_filter_ctl_7::Register>),
        (0x64 => adc_chn_val_0: ReadOnly<u32, adc_chn_val_0::Register>),
        (0x68 => adc_chn_val_1: ReadOnly<u32, adc_chn_val_1::Register>),
        (0x6c => adc_wakeup_ctl: ReadWrite<u32, adc_wakeup_ctl::Register>),
        (0x70 => filter_status: ReadWrite<u32, filter_status::Register>),
        (0x74 => adc_intr_ctl: ReadWrite<u32, adc_intr_ctl::Register>),
        (0x78 => adc_intr_status: ReadWrite<u32, adc_intr_status::Register>),
    }
}

register_bitfields![u32,
    INTR_STATE [
        DEBUG_CABLE OFFSET(0) NUMBITS(1) [],
    ],
    INTR_ENABLE [
        DEBUG_CABLE OFFSET(0) NUMBITS(1) [],
    ],
    INTR_TEST [
        DEBUG_CABLE OFFSET(0) NUMBITS(1) [],
    ],
    ALERT_TEST [
        FATAL_FAULT OFFSET(0) NUMBITS(1) [],
    ],
    ADC_EN_CTL [
        ADC_ENABLE OFFSET(0) NUMBITS(1) [],
        ONESHOT_MODE OFFSET(1) NUMBITS(1) [],
    ],
    ADC_PD_CTL [
        LP_MODE OFFSET(0) NUMBITS(1) [],
        PWRUP_TIME OFFSET(4) NUMBITS(4) [],
        WAKEUP_TIME OFFSET(8) NUMBITS(24) [],
    ],
    ADC_LP_SAMPLE_CTL [
        LP_SAMPLE_CNT OFFSET(0) NUMBITS(8) [],
    ],
    ADC_SAMPLE_CTL [
        NP_SAMPLE_CNT OFFSET(0) NUMBITS(16) [],
    ],
    ADC_FSM_RST [
        RST_EN OFFSET(0) NUMBITS(1) [],
    ],
    ADC_CHN0_FILTER_CTL_0 [
        MIN_V_0 OFFSET(2) NUMBITS(10) [],
        COND_0 OFFSET(12) NUMBITS(1) [],
        MAX_V_0 OFFSET(18) NUMBITS(10) [],
        EN_0 OFFSET(31) NUMBITS(1) [],
    ],
    ADC_CHN0_FILTER_CTL_1 [
        MIN_V_1 OFFSET(2) NUMBITS(10) [],
        COND_1 OFFSET(12) NUMBITS(1) [],
        MAX_V_1 OFFSET(18) NUMBITS(10) [],
        EN_1 OFFSET(31) NUMBITS(1) [],
    ],
    ADC_CHN0_FILTER_CTL_2 [
        MIN_V_2 OFFSET(2) NUMBITS(10) [],
        COND_2 OFFSET(12) NUMBITS(1) [],
        MAX_V_2 OFFSET(18) NUMBITS(10) [],
        EN_2 OFFSET(31) NUMBITS(1) [],
    ],
    ADC_CHN0_FILTER_CTL_3 [
        MIN_V_3 OFFSET(2) NUMBITS(10) [],
        COND_3 OFFSET(12) NUMBITS(1) [],
        MAX_V_3 OFFSET(18) NUMBITS(10) [],
        EN_3 OFFSET(31) NUMBITS(1) [],
    ],
    ADC_CHN0_FILTER_CTL_4 [
        MIN_V_4 OFFSET(2) NUMBITS(10) [],
        COND_4 OFFSET(12) NUMBITS(1) [],
        MAX_V_4 OFFSET(18) NUMBITS(10) [],
        EN_4 OFFSET(31) NUMBITS(1) [],
    ],
    ADC_CHN0_FILTER_CTL_5 [
        MIN_V_5 OFFSET(2) NUMBITS(10) [],
        COND_5 OFFSET(12) NUMBITS(1) [],
        MAX_V_5 OFFSET(18) NUMBITS(10) [],
        EN_5 OFFSET(31) NUMBITS(1) [],
    ],
    ADC_CHN0_FILTER_CTL_6 [
        MIN_V_6 OFFSET(2) NUMBITS(10) [],
        COND_6 OFFSET(12) NUMBITS(1) [],
        MAX_V_6 OFFSET(18) NUMBITS(10) [],
        EN_6 OFFSET(31) NUMBITS(1) [],
    ],
    ADC_CHN0_FILTER_CTL_7 [
        MIN_V_7 OFFSET(2) NUMBITS(10) [],
        COND_7 OFFSET(12) NUMBITS(1) [],
        MAX_V_7 OFFSET(18) NUMBITS(10) [],
        EN_7 OFFSET(31) NUMBITS(1) [],
    ],
    ADC_CHN1_FILTER_CTL_0 [
        MIN_V_0 OFFSET(2) NUMBITS(10) [],
        COND_0 OFFSET(12) NUMBITS(1) [],
        MAX_V_0 OFFSET(18) NUMBITS(10) [],
        EN_0 OFFSET(31) NUMBITS(1) [],
    ],
    ADC_CHN1_FILTER_CTL_1 [
        MIN_V_1 OFFSET(2) NUMBITS(10) [],
        COND_1 OFFSET(12) NUMBITS(1) [],
        MAX_V_1 OFFSET(18) NUMBITS(10) [],
        EN_1 OFFSET(31) NUMBITS(1) [],
    ],
    ADC_CHN1_FILTER_CTL_2 [
        MIN_V_2 OFFSET(2) NUMBITS(10) [],
        COND_2 OFFSET(12) NUMBITS(1) [],
        MAX_V_2 OFFSET(18) NUMBITS(10) [],
        EN_2 OFFSET(31) NUMBITS(1) [],
    ],
    ADC_CHN1_FILTER_CTL_3 [
        MIN_V_3 OFFSET(2) NUMBITS(10) [],
        COND_3 OFFSET(12) NUMBITS(1) [],
        MAX_V_3 OFFSET(18) NUMBITS(10) [],
        EN_3 OFFSET(31) NUMBITS(1) [],
    ],
    ADC_CHN1_FILTER_CTL_4 [
        MIN_V_4 OFFSET(2) NUMBITS(10) [],
        COND_4 OFFSET(12) NUMBITS(1) [],
        MAX_V_4 OFFSET(18) NUMBITS(10) [],
        EN_4 OFFSET(31) NUMBITS(1) [],
    ],
    ADC_CHN1_FILTER_CTL_5 [
        MIN_V_5 OFFSET(2) NUMBITS(10) [],
        COND_5 OFFSET(12) NUMBITS(1) [],
        MAX_V_5 OFFSET(18) NUMBITS(10) [],
        EN_5 OFFSET(31) NUMBITS(1) [],
    ],
    ADC_CHN1_FILTER_CTL_6 [
        MIN_V_6 OFFSET(2) NUMBITS(10) [],
        COND_6 OFFSET(12) NUMBITS(1) [],
        MAX_V_6 OFFSET(18) NUMBITS(10) [],
        EN_6 OFFSET(31) NUMBITS(1) [],
    ],
    ADC_CHN1_FILTER_CTL_7 [
        MIN_V_7 OFFSET(2) NUMBITS(10) [],
        COND_7 OFFSET(12) NUMBITS(1) [],
        MAX_V_7 OFFSET(18) NUMBITS(10) [],
        EN_7 OFFSET(31) NUMBITS(1) [],
    ],
    ADC_CHN_VAL_0 [
        ADC_CHN_VALUE_EXT_0 OFFSET(0) NUMBITS(2) [],
        ADC_CHN_VALUE_0 OFFSET(2) NUMBITS(10) [],
        ADC_CHN_VALUE_INTR_EXT_0 OFFSET(16) NUMBITS(2) [],
        ADC_CHN_VALUE_INTR_0 OFFSET(18) NUMBITS(10) [],
    ],
    ADC_CHN_VAL_1 [
        ADC_CHN_VALUE_EXT_1 OFFSET(0) NUMBITS(2) [],
        ADC_CHN_VALUE_1 OFFSET(2) NUMBITS(10) [],
        ADC_CHN_VALUE_INTR_EXT_1 OFFSET(16) NUMBITS(2) [],
        ADC_CHN_VALUE_INTR_1 OFFSET(18) NUMBITS(10) [],
    ],
    ADC_WAKEUP_CTL [
        EN OFFSET(0) NUMBITS(8) [],
    ],
    FILTER_STATUS [
        COND OFFSET(0) NUMBITS(8) [],
    ],
    ADC_INTR_CTL [
        EN OFFSET(0) NUMBITS(9) [],
    ],
    ADC_INTR_STATUS [
        CC_SINK_DET OFFSET(0) NUMBITS(1) [],
        CC_1A5_SINK_DET OFFSET(1) NUMBITS(1) [],
        CC_3A0_SINK_DET OFFSET(2) NUMBITS(1) [],
        CC_SRC_DET OFFSET(3) NUMBITS(1) [],
        CC_1A5_SRC_DET OFFSET(4) NUMBITS(1) [],
        CC_SRC_DET_FLIP OFFSET(5) NUMBITS(1) [],
        CC_1A5_SRC_DET_FLIP OFFSET(6) NUMBITS(1) [],
        CC_DISCON OFFSET(7) NUMBITS(1) [],
        ONESHOT OFFSET(8) NUMBITS(1) [],
    ],
];

// Number for ADC filters
pub const ADC_CTRL_PARAM_NUM_ADC_FILTER: u32 = 8;

// Number for ADC channels
pub const ADC_CTRL_PARAM_NUM_ADC_CHANNEL: u32 = 2;

// Number of alerts
pub const ADC_CTRL_PARAM_NUM_ALERTS: u32 = 1;

// Register width
pub const ADC_CTRL_PARAM_REG_WIDTH: u32 = 32;

// End generated register constants for adc_ctrl

