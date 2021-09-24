// Generated register struct for ALERT_HANDLER

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
    pub Alert_HandlerRegisters {
        (0x0 => intr_state: ReadWrite<u32, INTR_STATE::Register>),
        (0x4 => intr_enable: ReadWrite<u32, INTR_ENABLE::Register>),
        (0x8 => intr_test: WriteOnly<u32, INTR_TEST::Register>),
        (0xc => ping_timer_regwen: ReadWrite<u32, PING_TIMER_REGWEN::Register>),
        (0x10 => ping_timeout_cyc_shadowed: ReadWrite<u32, PING_TIMEOUT_CYC_SHADOWED::Register>),
        (0x14 => ping_timer_en_shadowed: ReadWrite<u32, PING_TIMER_EN_SHADOWED::Register>),
        (0x18 => alert_regwen_0: ReadWrite<u32, ALERT_REGWEN_0::Register>),
        (0x1c => alert_regwen_1: ReadWrite<u32, ALERT_REGWEN_1::Register>),
        (0x20 => alert_regwen_2: ReadWrite<u32, ALERT_REGWEN_2::Register>),
        (0x24 => alert_regwen_3: ReadWrite<u32, ALERT_REGWEN_3::Register>),
        (0x28 => alert_en_shadowed_0: ReadWrite<u32, ALERT_EN_SHADOWED_0::Register>),
        (0x2c => alert_en_shadowed_1: ReadWrite<u32, ALERT_EN_SHADOWED_1::Register>),
        (0x30 => alert_en_shadowed_2: ReadWrite<u32, ALERT_EN_SHADOWED_2::Register>),
        (0x34 => alert_en_shadowed_3: ReadWrite<u32, ALERT_EN_SHADOWED_3::Register>),
        (0x38 => alert_class_shadowed_0: ReadWrite<u32, ALERT_CLASS_SHADOWED_0::Register>),
        (0x3c => alert_class_shadowed_1: ReadWrite<u32, ALERT_CLASS_SHADOWED_1::Register>),
        (0x40 => alert_class_shadowed_2: ReadWrite<u32, ALERT_CLASS_SHADOWED_2::Register>),
        (0x44 => alert_class_shadowed_3: ReadWrite<u32, ALERT_CLASS_SHADOWED_3::Register>),
        (0x48 => alert_cause_0: ReadWrite<u32, ALERT_CAUSE_0::Register>),
        (0x4c => alert_cause_1: ReadWrite<u32, ALERT_CAUSE_1::Register>),
        (0x50 => alert_cause_2: ReadWrite<u32, ALERT_CAUSE_2::Register>),
        (0x54 => alert_cause_3: ReadWrite<u32, ALERT_CAUSE_3::Register>),
        (0x58 => loc_alert_regwen_0: ReadWrite<u32, LOC_ALERT_REGWEN_0::Register>),
        (0x5c => loc_alert_regwen_1: ReadWrite<u32, LOC_ALERT_REGWEN_1::Register>),
        (0x60 => loc_alert_regwen_2: ReadWrite<u32, LOC_ALERT_REGWEN_2::Register>),
        (0x64 => loc_alert_regwen_3: ReadWrite<u32, LOC_ALERT_REGWEN_3::Register>),
        (0x68 => loc_alert_regwen_4: ReadWrite<u32, LOC_ALERT_REGWEN_4::Register>),
        (0x6c => loc_alert_regwen_5: ReadWrite<u32, LOC_ALERT_REGWEN_5::Register>),
        (0x70 => loc_alert_regwen_6: ReadWrite<u32, LOC_ALERT_REGWEN_6::Register>),
        (0x74 => loc_alert_en_shadowed_0: ReadWrite<u32, LOC_ALERT_EN_SHADOWED_0::Register>),
        (0x78 => loc_alert_en_shadowed_1: ReadWrite<u32, LOC_ALERT_EN_SHADOWED_1::Register>),
        (0x7c => loc_alert_en_shadowed_2: ReadWrite<u32, LOC_ALERT_EN_SHADOWED_2::Register>),
        (0x80 => loc_alert_en_shadowed_3: ReadWrite<u32, LOC_ALERT_EN_SHADOWED_3::Register>),
        (0x84 => loc_alert_en_shadowed_4: ReadWrite<u32, LOC_ALERT_EN_SHADOWED_4::Register>),
        (0x88 => loc_alert_en_shadowed_5: ReadWrite<u32, LOC_ALERT_EN_SHADOWED_5::Register>),
        (0x8c => loc_alert_en_shadowed_6: ReadWrite<u32, LOC_ALERT_EN_SHADOWED_6::Register>),
        (0x90 => loc_alert_class_shadowed_0: ReadWrite<u32, LOC_ALERT_CLASS_SHADOWED_0::Register>),
        (0x94 => loc_alert_class_shadowed_1: ReadWrite<u32, LOC_ALERT_CLASS_SHADOWED_1::Register>),
        (0x98 => loc_alert_class_shadowed_2: ReadWrite<u32, LOC_ALERT_CLASS_SHADOWED_2::Register>),
        (0x9c => loc_alert_class_shadowed_3: ReadWrite<u32, LOC_ALERT_CLASS_SHADOWED_3::Register>),
        (0xa0 => loc_alert_class_shadowed_4: ReadWrite<u32, LOC_ALERT_CLASS_SHADOWED_4::Register>),
        (0xa4 => loc_alert_class_shadowed_5: ReadWrite<u32, LOC_ALERT_CLASS_SHADOWED_5::Register>),
        (0xa8 => loc_alert_class_shadowed_6: ReadWrite<u32, LOC_ALERT_CLASS_SHADOWED_6::Register>),
        (0xac => loc_alert_cause_0: ReadWrite<u32, LOC_ALERT_CAUSE_0::Register>),
        (0xb0 => loc_alert_cause_1: ReadWrite<u32, LOC_ALERT_CAUSE_1::Register>),
        (0xb4 => loc_alert_cause_2: ReadWrite<u32, LOC_ALERT_CAUSE_2::Register>),
        (0xb8 => loc_alert_cause_3: ReadWrite<u32, LOC_ALERT_CAUSE_3::Register>),
        (0xbc => loc_alert_cause_4: ReadWrite<u32, LOC_ALERT_CAUSE_4::Register>),
        (0xc0 => loc_alert_cause_5: ReadWrite<u32, LOC_ALERT_CAUSE_5::Register>),
        (0xc4 => loc_alert_cause_6: ReadWrite<u32, LOC_ALERT_CAUSE_6::Register>),
        (0xc8 => classa_regwen: ReadWrite<u32, CLASSA_REGWEN::Register>),
        (0xcc => classa_ctrl_shadowed: ReadWrite<u32, CLASSA_CTRL_SHADOWED::Register>),
        (0xd0 => classa_clr_regwen: ReadWrite<u32, CLASSA_CLR_REGWEN::Register>),
        (0xd4 => classa_clr_shadowed: ReadWrite<u32, CLASSA_CLR_SHADOWED::Register>),
        (0xd8 => classa_accum_cnt: ReadOnly<u32, CLASSA_ACCUM_CNT::Register>),
        (0xdc => classa_accum_thresh_shadowed: ReadWrite<u32, CLASSA_ACCUM_THRESH_SHADOWED::Register>),
        (0xe0 => classa_timeout_cyc_shadowed: ReadWrite<u32, CLASSA_TIMEOUT_CYC_SHADOWED::Register>),
        (0xe4 => classa_crashdump_trigger_shadowed: ReadWrite<u32, CLASSA_CRASHDUMP_TRIGGER_SHADOWED::Register>),
        (0xe8 => classa_phase0_cyc_shadowed: ReadWrite<u32, CLASSA_PHASE0_CYC_SHADOWED::Register>),
        (0xec => classa_phase1_cyc_shadowed: ReadWrite<u32, CLASSA_PHASE1_CYC_SHADOWED::Register>),
        (0xf0 => classa_phase2_cyc_shadowed: ReadWrite<u32, CLASSA_PHASE2_CYC_SHADOWED::Register>),
        (0xf4 => classa_phase3_cyc_shadowed: ReadWrite<u32, CLASSA_PHASE3_CYC_SHADOWED::Register>),
        (0xf8 => classa_esc_cnt: ReadOnly<u32, CLASSA_ESC_CNT::Register>),
        (0xfc => classa_state: ReadOnly<u32, CLASSA_STATE::Register>),
        (0x100 => classb_regwen: ReadWrite<u32, CLASSB_REGWEN::Register>),
        (0x104 => classb_ctrl_shadowed: ReadWrite<u32, CLASSB_CTRL_SHADOWED::Register>),
        (0x108 => classb_clr_regwen: ReadWrite<u32, CLASSB_CLR_REGWEN::Register>),
        (0x10c => classb_clr_shadowed: ReadWrite<u32, CLASSB_CLR_SHADOWED::Register>),
        (0x110 => classb_accum_cnt: ReadOnly<u32, CLASSB_ACCUM_CNT::Register>),
        (0x114 => classb_accum_thresh_shadowed: ReadWrite<u32, CLASSB_ACCUM_THRESH_SHADOWED::Register>),
        (0x118 => classb_timeout_cyc_shadowed: ReadWrite<u32, CLASSB_TIMEOUT_CYC_SHADOWED::Register>),
        (0x11c => classb_crashdump_trigger_shadowed: ReadWrite<u32, CLASSB_CRASHDUMP_TRIGGER_SHADOWED::Register>),
        (0x120 => classb_phase0_cyc_shadowed: ReadWrite<u32, CLASSB_PHASE0_CYC_SHADOWED::Register>),
        (0x124 => classb_phase1_cyc_shadowed: ReadWrite<u32, CLASSB_PHASE1_CYC_SHADOWED::Register>),
        (0x128 => classb_phase2_cyc_shadowed: ReadWrite<u32, CLASSB_PHASE2_CYC_SHADOWED::Register>),
        (0x12c => classb_phase3_cyc_shadowed: ReadWrite<u32, CLASSB_PHASE3_CYC_SHADOWED::Register>),
        (0x130 => classb_esc_cnt: ReadOnly<u32, CLASSB_ESC_CNT::Register>),
        (0x134 => classb_state: ReadOnly<u32, CLASSB_STATE::Register>),
        (0x138 => classc_regwen: ReadWrite<u32, CLASSC_REGWEN::Register>),
        (0x13c => classc_ctrl_shadowed: ReadWrite<u32, CLASSC_CTRL_SHADOWED::Register>),
        (0x140 => classc_clr_regwen: ReadWrite<u32, CLASSC_CLR_REGWEN::Register>),
        (0x144 => classc_clr_shadowed: ReadWrite<u32, CLASSC_CLR_SHADOWED::Register>),
        (0x148 => classc_accum_cnt: ReadOnly<u32, CLASSC_ACCUM_CNT::Register>),
        (0x14c => classc_accum_thresh_shadowed: ReadWrite<u32, CLASSC_ACCUM_THRESH_SHADOWED::Register>),
        (0x150 => classc_timeout_cyc_shadowed: ReadWrite<u32, CLASSC_TIMEOUT_CYC_SHADOWED::Register>),
        (0x154 => classc_crashdump_trigger_shadowed: ReadWrite<u32, CLASSC_CRASHDUMP_TRIGGER_SHADOWED::Register>),
        (0x158 => classc_phase0_cyc_shadowed: ReadWrite<u32, CLASSC_PHASE0_CYC_SHADOWED::Register>),
        (0x15c => classc_phase1_cyc_shadowed: ReadWrite<u32, CLASSC_PHASE1_CYC_SHADOWED::Register>),
        (0x160 => classc_phase2_cyc_shadowed: ReadWrite<u32, CLASSC_PHASE2_CYC_SHADOWED::Register>),
        (0x164 => classc_phase3_cyc_shadowed: ReadWrite<u32, CLASSC_PHASE3_CYC_SHADOWED::Register>),
        (0x168 => classc_esc_cnt: ReadOnly<u32, CLASSC_ESC_CNT::Register>),
        (0x16c => classc_state: ReadOnly<u32, CLASSC_STATE::Register>),
        (0x170 => classd_regwen: ReadWrite<u32, CLASSD_REGWEN::Register>),
        (0x174 => classd_ctrl_shadowed: ReadWrite<u32, CLASSD_CTRL_SHADOWED::Register>),
        (0x178 => classd_clr_regwen: ReadWrite<u32, CLASSD_CLR_REGWEN::Register>),
        (0x17c => classd_clr_shadowed: ReadWrite<u32, CLASSD_CLR_SHADOWED::Register>),
        (0x180 => classd_accum_cnt: ReadOnly<u32, CLASSD_ACCUM_CNT::Register>),
        (0x184 => classd_accum_thresh_shadowed: ReadWrite<u32, CLASSD_ACCUM_THRESH_SHADOWED::Register>),
        (0x188 => classd_timeout_cyc_shadowed: ReadWrite<u32, CLASSD_TIMEOUT_CYC_SHADOWED::Register>),
        (0x18c => classd_crashdump_trigger_shadowed: ReadWrite<u32, CLASSD_CRASHDUMP_TRIGGER_SHADOWED::Register>),
        (0x190 => classd_phase0_cyc_shadowed: ReadWrite<u32, CLASSD_PHASE0_CYC_SHADOWED::Register>),
        (0x194 => classd_phase1_cyc_shadowed: ReadWrite<u32, CLASSD_PHASE1_CYC_SHADOWED::Register>),
        (0x198 => classd_phase2_cyc_shadowed: ReadWrite<u32, CLASSD_PHASE2_CYC_SHADOWED::Register>),
        (0x19c => classd_phase3_cyc_shadowed: ReadWrite<u32, CLASSD_PHASE3_CYC_SHADOWED::Register>),
        (0x1a0 => classd_esc_cnt: ReadOnly<u32, CLASSD_ESC_CNT::Register>),
        (0x1a4 => classd_state: ReadOnly<u32, CLASSD_STATE::Register>),
    }
}

register_bitfields![u32,
    INTR_STATE [
        CLASSA OFFSET(0) NUMBITS(1) [],
        CLASSB OFFSET(1) NUMBITS(1) [],
        CLASSC OFFSET(2) NUMBITS(1) [],
        CLASSD OFFSET(3) NUMBITS(1) [],
    ],
    INTR_ENABLE [
        CLASSA OFFSET(0) NUMBITS(1) [],
        CLASSB OFFSET(1) NUMBITS(1) [],
        CLASSC OFFSET(2) NUMBITS(1) [],
        CLASSD OFFSET(3) NUMBITS(1) [],
    ],
    INTR_TEST [
        CLASSA OFFSET(0) NUMBITS(1) [],
        CLASSB OFFSET(1) NUMBITS(1) [],
        CLASSC OFFSET(2) NUMBITS(1) [],
        CLASSD OFFSET(3) NUMBITS(1) [],
    ],
    PING_TIMER_REGWEN [
        PING_TIMER_REGWEN OFFSET(0) NUMBITS(1) [],
    ],
    PING_TIMEOUT_CYC_SHADOWED [
        PING_TIMEOUT_CYC_SHADOWED OFFSET(0) NUMBITS(16) [],
    ],
    PING_TIMER_EN_SHADOWED [
        PING_TIMER_EN_SHADOWED OFFSET(0) NUMBITS(1) [],
    ],
    ALERT_REGWEN_0 [
        EN_0 OFFSET(0) NUMBITS(1) [],
    ],
    ALERT_REGWEN_1 [
        EN_1 OFFSET(0) NUMBITS(1) [],
    ],
    ALERT_REGWEN_2 [
        EN_2 OFFSET(0) NUMBITS(1) [],
    ],
    ALERT_REGWEN_3 [
        EN_3 OFFSET(0) NUMBITS(1) [],
    ],
    ALERT_EN_SHADOWED_0 [
        EN_A_0 OFFSET(0) NUMBITS(1) [],
    ],
    ALERT_EN_SHADOWED_1 [
        EN_A_1 OFFSET(0) NUMBITS(1) [],
    ],
    ALERT_EN_SHADOWED_2 [
        EN_A_2 OFFSET(0) NUMBITS(1) [],
    ],
    ALERT_EN_SHADOWED_3 [
        EN_A_3 OFFSET(0) NUMBITS(1) [],
    ],
    ALERT_CLASS_SHADOWED_0 [
        CLASS_A_0 OFFSET(0) NUMBITS(2) [
            CLASSA = 0,
            CLASSB = 1,
            CLASSC = 2,
            CLASSD = 3,
        ],
    ],
    ALERT_CLASS_SHADOWED_1 [
        CLASS_A_1 OFFSET(0) NUMBITS(2) [],
    ],
    ALERT_CLASS_SHADOWED_2 [
        CLASS_A_2 OFFSET(0) NUMBITS(2) [],
    ],
    ALERT_CLASS_SHADOWED_3 [
        CLASS_A_3 OFFSET(0) NUMBITS(2) [],
    ],
    ALERT_CAUSE_0 [
        A_0 OFFSET(0) NUMBITS(1) [],
    ],
    ALERT_CAUSE_1 [
        A_1 OFFSET(0) NUMBITS(1) [],
    ],
    ALERT_CAUSE_2 [
        A_2 OFFSET(0) NUMBITS(1) [],
    ],
    ALERT_CAUSE_3 [
        A_3 OFFSET(0) NUMBITS(1) [],
    ],
    LOC_ALERT_REGWEN_0 [
        EN_0 OFFSET(0) NUMBITS(1) [],
    ],
    LOC_ALERT_REGWEN_1 [
        EN_1 OFFSET(0) NUMBITS(1) [],
    ],
    LOC_ALERT_REGWEN_2 [
        EN_2 OFFSET(0) NUMBITS(1) [],
    ],
    LOC_ALERT_REGWEN_3 [
        EN_3 OFFSET(0) NUMBITS(1) [],
    ],
    LOC_ALERT_REGWEN_4 [
        EN_4 OFFSET(0) NUMBITS(1) [],
    ],
    LOC_ALERT_REGWEN_5 [
        EN_5 OFFSET(0) NUMBITS(1) [],
    ],
    LOC_ALERT_REGWEN_6 [
        EN_6 OFFSET(0) NUMBITS(1) [],
    ],
    LOC_ALERT_EN_SHADOWED_0 [
        EN_LA_0 OFFSET(0) NUMBITS(1) [],
    ],
    LOC_ALERT_EN_SHADOWED_1 [
        EN_LA_1 OFFSET(0) NUMBITS(1) [],
    ],
    LOC_ALERT_EN_SHADOWED_2 [
        EN_LA_2 OFFSET(0) NUMBITS(1) [],
    ],
    LOC_ALERT_EN_SHADOWED_3 [
        EN_LA_3 OFFSET(0) NUMBITS(1) [],
    ],
    LOC_ALERT_EN_SHADOWED_4 [
        EN_LA_4 OFFSET(0) NUMBITS(1) [],
    ],
    LOC_ALERT_EN_SHADOWED_5 [
        EN_LA_5 OFFSET(0) NUMBITS(1) [],
    ],
    LOC_ALERT_EN_SHADOWED_6 [
        EN_LA_6 OFFSET(0) NUMBITS(1) [],
    ],
    LOC_ALERT_CLASS_SHADOWED_0 [
        CLASS_LA_0 OFFSET(0) NUMBITS(2) [
            CLASSA = 0,
            CLASSB = 1,
            CLASSC = 2,
            CLASSD = 3,
        ],
    ],
    LOC_ALERT_CLASS_SHADOWED_1 [
        CLASS_LA_1 OFFSET(0) NUMBITS(2) [],
    ],
    LOC_ALERT_CLASS_SHADOWED_2 [
        CLASS_LA_2 OFFSET(0) NUMBITS(2) [],
    ],
    LOC_ALERT_CLASS_SHADOWED_3 [
        CLASS_LA_3 OFFSET(0) NUMBITS(2) [],
    ],
    LOC_ALERT_CLASS_SHADOWED_4 [
        CLASS_LA_4 OFFSET(0) NUMBITS(2) [],
    ],
    LOC_ALERT_CLASS_SHADOWED_5 [
        CLASS_LA_5 OFFSET(0) NUMBITS(2) [],
    ],
    LOC_ALERT_CLASS_SHADOWED_6 [
        CLASS_LA_6 OFFSET(0) NUMBITS(2) [],
    ],
    LOC_ALERT_CAUSE_0 [
        LA_0 OFFSET(0) NUMBITS(1) [],
    ],
    LOC_ALERT_CAUSE_1 [
        LA_1 OFFSET(0) NUMBITS(1) [],
    ],
    LOC_ALERT_CAUSE_2 [
        LA_2 OFFSET(0) NUMBITS(1) [],
    ],
    LOC_ALERT_CAUSE_3 [
        LA_3 OFFSET(0) NUMBITS(1) [],
    ],
    LOC_ALERT_CAUSE_4 [
        LA_4 OFFSET(0) NUMBITS(1) [],
    ],
    LOC_ALERT_CAUSE_5 [
        LA_5 OFFSET(0) NUMBITS(1) [],
    ],
    LOC_ALERT_CAUSE_6 [
        LA_6 OFFSET(0) NUMBITS(1) [],
    ],
    CLASSA_REGWEN [
        CLASSA_REGWEN OFFSET(0) NUMBITS(1) [],
    ],
    CLASSA_CTRL_SHADOWED [
        EN OFFSET(0) NUMBITS(1) [],
        LOCK OFFSET(1) NUMBITS(1) [],
        EN_E0 OFFSET(2) NUMBITS(1) [],
        EN_E1 OFFSET(3) NUMBITS(1) [],
        EN_E2 OFFSET(4) NUMBITS(1) [],
        EN_E3 OFFSET(5) NUMBITS(1) [],
        MAP_E0 OFFSET(6) NUMBITS(2) [],
        MAP_E1 OFFSET(8) NUMBITS(2) [],
        MAP_E2 OFFSET(10) NUMBITS(2) [],
        MAP_E3 OFFSET(12) NUMBITS(2) [],
    ],
    CLASSA_CLR_REGWEN [
        CLASSA_CLR_REGWEN OFFSET(0) NUMBITS(1) [],
    ],
    CLASSA_CLR_SHADOWED [
        CLASSA_CLR_SHADOWED OFFSET(0) NUMBITS(1) [],
    ],
    CLASSA_ACCUM_CNT [
        CLASSA_ACCUM_CNT OFFSET(0) NUMBITS(16) [],
    ],
    CLASSA_ACCUM_THRESH_SHADOWED [
        CLASSA_ACCUM_THRESH_SHADOWED OFFSET(0) NUMBITS(16) [],
    ],
    CLASSA_TIMEOUT_CYC_SHADOWED [
        CLASSA_TIMEOUT_CYC_SHADOWED OFFSET(0) NUMBITS(32) [],
    ],
    CLASSA_CRASHDUMP_TRIGGER_SHADOWED [
        CLASSA_CRASHDUMP_TRIGGER_SHADOWED OFFSET(0) NUMBITS(2) [],
    ],
    CLASSA_PHASE0_CYC_SHADOWED [
        CLASSA_PHASE0_CYC_SHADOWED OFFSET(0) NUMBITS(32) [],
    ],
    CLASSA_PHASE1_CYC_SHADOWED [
        CLASSA_PHASE1_CYC_SHADOWED OFFSET(0) NUMBITS(32) [],
    ],
    CLASSA_PHASE2_CYC_SHADOWED [
        CLASSA_PHASE2_CYC_SHADOWED OFFSET(0) NUMBITS(32) [],
    ],
    CLASSA_PHASE3_CYC_SHADOWED [
        CLASSA_PHASE3_CYC_SHADOWED OFFSET(0) NUMBITS(32) [],
    ],
    CLASSA_ESC_CNT [
        CLASSA_ESC_CNT OFFSET(0) NUMBITS(32) [],
    ],
    CLASSA_STATE [
        CLASSA_STATE OFFSET(0) NUMBITS(3) [
            IDLE = 0,
            TIMEOUT = 1,
            FSMERROR = 2,
            TERMINAL = 3,
            PHASE0 = 4,
            PHASE1 = 5,
            PHASE2 = 6,
            PHASE3 = 7,
        ],
    ],
    CLASSB_REGWEN [
        CLASSB_REGWEN OFFSET(0) NUMBITS(1) [],
    ],
    CLASSB_CTRL_SHADOWED [
        EN OFFSET(0) NUMBITS(1) [],
        LOCK OFFSET(1) NUMBITS(1) [],
        EN_E0 OFFSET(2) NUMBITS(1) [],
        EN_E1 OFFSET(3) NUMBITS(1) [],
        EN_E2 OFFSET(4) NUMBITS(1) [],
        EN_E3 OFFSET(5) NUMBITS(1) [],
        MAP_E0 OFFSET(6) NUMBITS(2) [],
        MAP_E1 OFFSET(8) NUMBITS(2) [],
        MAP_E2 OFFSET(10) NUMBITS(2) [],
        MAP_E3 OFFSET(12) NUMBITS(2) [],
    ],
    CLASSB_CLR_REGWEN [
        CLASSB_CLR_REGWEN OFFSET(0) NUMBITS(1) [],
    ],
    CLASSB_CLR_SHADOWED [
        CLASSB_CLR_SHADOWED OFFSET(0) NUMBITS(1) [],
    ],
    CLASSB_ACCUM_CNT [
        CLASSB_ACCUM_CNT OFFSET(0) NUMBITS(16) [],
    ],
    CLASSB_ACCUM_THRESH_SHADOWED [
        CLASSB_ACCUM_THRESH_SHADOWED OFFSET(0) NUMBITS(16) [],
    ],
    CLASSB_TIMEOUT_CYC_SHADOWED [
        CLASSB_TIMEOUT_CYC_SHADOWED OFFSET(0) NUMBITS(32) [],
    ],
    CLASSB_CRASHDUMP_TRIGGER_SHADOWED [
        CLASSB_CRASHDUMP_TRIGGER_SHADOWED OFFSET(0) NUMBITS(2) [],
    ],
    CLASSB_PHASE0_CYC_SHADOWED [
        CLASSB_PHASE0_CYC_SHADOWED OFFSET(0) NUMBITS(32) [],
    ],
    CLASSB_PHASE1_CYC_SHADOWED [
        CLASSB_PHASE1_CYC_SHADOWED OFFSET(0) NUMBITS(32) [],
    ],
    CLASSB_PHASE2_CYC_SHADOWED [
        CLASSB_PHASE2_CYC_SHADOWED OFFSET(0) NUMBITS(32) [],
    ],
    CLASSB_PHASE3_CYC_SHADOWED [
        CLASSB_PHASE3_CYC_SHADOWED OFFSET(0) NUMBITS(32) [],
    ],
    CLASSB_ESC_CNT [
        CLASSB_ESC_CNT OFFSET(0) NUMBITS(32) [],
    ],
    CLASSB_STATE [
        CLASSB_STATE OFFSET(0) NUMBITS(3) [
            IDLE = 0,
            TIMEOUT = 1,
            FSMERROR = 2,
            TERMINAL = 3,
            PHASE0 = 4,
            PHASE1 = 5,
            PHASE2 = 6,
            PHASE3 = 7,
        ],
    ],
    CLASSC_REGWEN [
        CLASSC_REGWEN OFFSET(0) NUMBITS(1) [],
    ],
    CLASSC_CTRL_SHADOWED [
        EN OFFSET(0) NUMBITS(1) [],
        LOCK OFFSET(1) NUMBITS(1) [],
        EN_E0 OFFSET(2) NUMBITS(1) [],
        EN_E1 OFFSET(3) NUMBITS(1) [],
        EN_E2 OFFSET(4) NUMBITS(1) [],
        EN_E3 OFFSET(5) NUMBITS(1) [],
        MAP_E0 OFFSET(6) NUMBITS(2) [],
        MAP_E1 OFFSET(8) NUMBITS(2) [],
        MAP_E2 OFFSET(10) NUMBITS(2) [],
        MAP_E3 OFFSET(12) NUMBITS(2) [],
    ],
    CLASSC_CLR_REGWEN [
        CLASSC_CLR_REGWEN OFFSET(0) NUMBITS(1) [],
    ],
    CLASSC_CLR_SHADOWED [
        CLASSC_CLR_SHADOWED OFFSET(0) NUMBITS(1) [],
    ],
    CLASSC_ACCUM_CNT [
        CLASSC_ACCUM_CNT OFFSET(0) NUMBITS(16) [],
    ],
    CLASSC_ACCUM_THRESH_SHADOWED [
        CLASSC_ACCUM_THRESH_SHADOWED OFFSET(0) NUMBITS(16) [],
    ],
    CLASSC_TIMEOUT_CYC_SHADOWED [
        CLASSC_TIMEOUT_CYC_SHADOWED OFFSET(0) NUMBITS(32) [],
    ],
    CLASSC_CRASHDUMP_TRIGGER_SHADOWED [
        CLASSC_CRASHDUMP_TRIGGER_SHADOWED OFFSET(0) NUMBITS(2) [],
    ],
    CLASSC_PHASE0_CYC_SHADOWED [
        CLASSC_PHASE0_CYC_SHADOWED OFFSET(0) NUMBITS(32) [],
    ],
    CLASSC_PHASE1_CYC_SHADOWED [
        CLASSC_PHASE1_CYC_SHADOWED OFFSET(0) NUMBITS(32) [],
    ],
    CLASSC_PHASE2_CYC_SHADOWED [
        CLASSC_PHASE2_CYC_SHADOWED OFFSET(0) NUMBITS(32) [],
    ],
    CLASSC_PHASE3_CYC_SHADOWED [
        CLASSC_PHASE3_CYC_SHADOWED OFFSET(0) NUMBITS(32) [],
    ],
    CLASSC_ESC_CNT [
        CLASSC_ESC_CNT OFFSET(0) NUMBITS(32) [],
    ],
    CLASSC_STATE [
        CLASSC_STATE OFFSET(0) NUMBITS(3) [
            IDLE = 0,
            TIMEOUT = 1,
            FSMERROR = 2,
            TERMINAL = 3,
            PHASE0 = 4,
            PHASE1 = 5,
            PHASE2 = 6,
            PHASE3 = 7,
        ],
    ],
    CLASSD_REGWEN [
        CLASSD_REGWEN OFFSET(0) NUMBITS(1) [],
    ],
    CLASSD_CTRL_SHADOWED [
        EN OFFSET(0) NUMBITS(1) [],
        LOCK OFFSET(1) NUMBITS(1) [],
        EN_E0 OFFSET(2) NUMBITS(1) [],
        EN_E1 OFFSET(3) NUMBITS(1) [],
        EN_E2 OFFSET(4) NUMBITS(1) [],
        EN_E3 OFFSET(5) NUMBITS(1) [],
        MAP_E0 OFFSET(6) NUMBITS(2) [],
        MAP_E1 OFFSET(8) NUMBITS(2) [],
        MAP_E2 OFFSET(10) NUMBITS(2) [],
        MAP_E3 OFFSET(12) NUMBITS(2) [],
    ],
    CLASSD_CLR_REGWEN [
        CLASSD_CLR_REGWEN OFFSET(0) NUMBITS(1) [],
    ],
    CLASSD_CLR_SHADOWED [
        CLASSD_CLR_SHADOWED OFFSET(0) NUMBITS(1) [],
    ],
    CLASSD_ACCUM_CNT [
        CLASSD_ACCUM_CNT OFFSET(0) NUMBITS(16) [],
    ],
    CLASSD_ACCUM_THRESH_SHADOWED [
        CLASSD_ACCUM_THRESH_SHADOWED OFFSET(0) NUMBITS(16) [],
    ],
    CLASSD_TIMEOUT_CYC_SHADOWED [
        CLASSD_TIMEOUT_CYC_SHADOWED OFFSET(0) NUMBITS(32) [],
    ],
    CLASSD_CRASHDUMP_TRIGGER_SHADOWED [
        CLASSD_CRASHDUMP_TRIGGER_SHADOWED OFFSET(0) NUMBITS(2) [],
    ],
    CLASSD_PHASE0_CYC_SHADOWED [
        CLASSD_PHASE0_CYC_SHADOWED OFFSET(0) NUMBITS(32) [],
    ],
    CLASSD_PHASE1_CYC_SHADOWED [
        CLASSD_PHASE1_CYC_SHADOWED OFFSET(0) NUMBITS(32) [],
    ],
    CLASSD_PHASE2_CYC_SHADOWED [
        CLASSD_PHASE2_CYC_SHADOWED OFFSET(0) NUMBITS(32) [],
    ],
    CLASSD_PHASE3_CYC_SHADOWED [
        CLASSD_PHASE3_CYC_SHADOWED OFFSET(0) NUMBITS(32) [],
    ],
    CLASSD_ESC_CNT [
        CLASSD_ESC_CNT OFFSET(0) NUMBITS(32) [],
    ],
    CLASSD_STATE [
        CLASSD_STATE OFFSET(0) NUMBITS(3) [
            IDLE = 0,
            TIMEOUT = 1,
            FSMERROR = 2,
            TERMINAL = 3,
            PHASE0 = 4,
            PHASE1 = 5,
            PHASE2 = 6,
            PHASE3 = 7,
        ],
    ],
];

// Number of alert channels.
pub const ALERT_HANDLER_PARAM_N_ALERTS: u32 = 4;

// Width of the escalation timer.
pub const ALERT_HANDLER_PARAM_ESC_CNT_DW: u32 = 32;

// Width of the accumulation counter.
pub const ALERT_HANDLER_PARAM_ACCU_CNT_DW: u32 = 16;

// Number of classes
pub const ALERT_HANDLER_PARAM_N_CLASSES: u32 = 4;

// Number of escalation severities
pub const ALERT_HANDLER_PARAM_N_ESC_SEV: u32 = 4;

// Number of escalation phases
pub const ALERT_HANDLER_PARAM_N_PHASES: u32 = 4;

// Number of local alerts
pub const ALERT_HANDLER_PARAM_N_LOC_ALERT: u32 = 7;

// Width of ping counter
pub const ALERT_HANDLER_PARAM_PING_CNT_DW: u32 = 16;

// Width of phase ID
pub const ALERT_HANDLER_PARAM_PHASE_DW: u32 = 2;

// Width of class ID
pub const ALERT_HANDLER_PARAM_CLASS_DW: u32 = 2;

// Local alert ID for alert ping failure.
pub const ALERT_HANDLER_PARAM_LOCAL_ALERT_ID_ALERT_PINGFAIL: u32 = 0;

// Local alert ID for escalation ping failure.
pub const ALERT_HANDLER_PARAM_LOCAL_ALERT_ID_ESC_PINGFAIL: u32 = 1;

// Local alert ID for alert integrity failure.
pub const ALERT_HANDLER_PARAM_LOCAL_ALERT_ID_ALERT_INTEGFAIL: u32 = 2;

// Local alert ID for escalation integrity failure.
pub const ALERT_HANDLER_PARAM_LOCAL_ALERT_ID_ESC_INTEGFAIL: u32 = 3;

// Local alert ID for bus integrity failure.
pub const ALERT_HANDLER_PARAM_LOCAL_ALERT_ID_BUS_INTEGFAIL: u32 = 4;

// Local alert ID for shadow register update error.
pub const ALERT_HANDLER_PARAM_LOCAL_ALERT_ID_SHADOW_REG_UPDATE_ERROR: u32 = 5;

// Local alert ID for shadow register storage error.
pub const ALERT_HANDLER_PARAM_LOCAL_ALERT_ID_SHADOW_REG_STORAGE_ERROR: u32 = 6;

// Last local alert ID.
pub const ALERT_HANDLER_PARAM_LOCAL_ALERT_ID_LAST: u32 = 6;

// Register width
pub const ALERT_HANDLER_PARAM_REG_WIDTH: u32 = 32;

// Register write enable for alert enable bits. (common parameters)
pub const ALERT_HANDLER_ALERT_REGWEN_EN_FIELD_WIDTH: u32 = 1;
pub const ALERT_HANDLER_ALERT_REGWEN_EN_FIELDS_PER_REG: u32 = 32;
pub const ALERT_HANDLER_ALERT_REGWEN_MULTIREG_COUNT: u32 = 4;

// Enable register for alerts. (common parameters)
pub const ALERT_HANDLER_ALERT_EN_SHADOWED_EN_A_FIELD_WIDTH: u32 = 1;
pub const ALERT_HANDLER_ALERT_EN_SHADOWED_EN_A_FIELDS_PER_REG: u32 = 32;
pub const ALERT_HANDLER_ALERT_EN_SHADOWED_MULTIREG_COUNT: u32 = 4;

// Class assignment of alerts. (common parameters)
pub const ALERT_HANDLER_ALERT_CLASS_SHADOWED_CLASS_A_FIELD_WIDTH: u32 = 2;
pub const ALERT_HANDLER_ALERT_CLASS_SHADOWED_CLASS_A_FIELDS_PER_REG: u32 = 16;
pub const ALERT_HANDLER_ALERT_CLASS_SHADOWED_MULTIREG_COUNT: u32 = 4;

// Alert Cause Register (common parameters)
pub const ALERT_HANDLER_ALERT_CAUSE_A_FIELD_WIDTH: u32 = 1;
pub const ALERT_HANDLER_ALERT_CAUSE_A_FIELDS_PER_REG: u32 = 32;
pub const ALERT_HANDLER_ALERT_CAUSE_MULTIREG_COUNT: u32 = 4;

// Register write enable for alert enable bits. (common parameters)
pub const ALERT_HANDLER_LOC_ALERT_REGWEN_EN_FIELD_WIDTH: u32 = 1;
pub const ALERT_HANDLER_LOC_ALERT_REGWEN_EN_FIELDS_PER_REG: u32 = 32;
pub const ALERT_HANDLER_LOC_ALERT_REGWEN_MULTIREG_COUNT: u32 = 7;

// Enable register for the local alerts
pub const ALERT_HANDLER_LOC_ALERT_EN_SHADOWED_EN_LA_FIELD_WIDTH: u32 = 1;
pub const ALERT_HANDLER_LOC_ALERT_EN_SHADOWED_EN_LA_FIELDS_PER_REG: u32 = 32;
pub const ALERT_HANDLER_LOC_ALERT_EN_SHADOWED_MULTIREG_COUNT: u32 = 7;

// Class assignment of the local alerts
pub const ALERT_HANDLER_LOC_ALERT_CLASS_SHADOWED_CLASS_LA_FIELD_WIDTH: u32 = 2;
pub const ALERT_HANDLER_LOC_ALERT_CLASS_SHADOWED_CLASS_LA_FIELDS_PER_REG: u32 = 16;
pub const ALERT_HANDLER_LOC_ALERT_CLASS_SHADOWED_MULTIREG_COUNT: u32 = 7;

// Alert Cause Register for the local alerts
pub const ALERT_HANDLER_LOC_ALERT_CAUSE_LA_FIELD_WIDTH: u32 = 1;
pub const ALERT_HANDLER_LOC_ALERT_CAUSE_LA_FIELDS_PER_REG: u32 = 32;
pub const ALERT_HANDLER_LOC_ALERT_CAUSE_MULTIREG_COUNT: u32 = 7;

// End generated register constants for ALERT_HANDLER

