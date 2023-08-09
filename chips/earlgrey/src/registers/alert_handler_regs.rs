// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright lowRISC contributors 2023.

// Generated register constants for alert_handler.
// Built for Earlgrey-M2.5.1-RC1-438-gacc67de99
// https://github.com/lowRISC/opentitan/tree/acc67de992ee8de5f2481b1b9580679850d8b5f5
// Tree status: clean
// Build date: 2023-08-08T00:15:38

// Original reference file: hw/top_earlgrey/ip_autogen/alert_handler/data/alert_handler.hjson
use kernel::utilities::registers::ReadWrite;
use kernel::utilities::registers::{register_bitfields, register_structs};
/// Number of alert channels.
pub const ALERT_HANDLER_PARAM_N_ALERTS: u32 = 65;
/// Number of LPGs.
pub const ALERT_HANDLER_PARAM_N_LPG: u32 = 24;
/// Width of LPG ID.
pub const ALERT_HANDLER_PARAM_N_LPG_WIDTH: u32 = 5;
/// Width of the escalation timer.
pub const ALERT_HANDLER_PARAM_ESC_CNT_DW: u32 = 32;
/// Width of the accumulation counter.
pub const ALERT_HANDLER_PARAM_ACCU_CNT_DW: u32 = 16;
/// Number of classes
pub const ALERT_HANDLER_PARAM_N_CLASSES: u32 = 4;
/// Number of escalation severities
pub const ALERT_HANDLER_PARAM_N_ESC_SEV: u32 = 4;
/// Number of escalation phases
pub const ALERT_HANDLER_PARAM_N_PHASES: u32 = 4;
/// Number of local alerts
pub const ALERT_HANDLER_PARAM_N_LOC_ALERT: u32 = 7;
/// Width of ping counter
pub const ALERT_HANDLER_PARAM_PING_CNT_DW: u32 = 16;
/// Width of phase ID
pub const ALERT_HANDLER_PARAM_PHASE_DW: u32 = 2;
/// Width of class ID
pub const ALERT_HANDLER_PARAM_CLASS_DW: u32 = 2;
/// Local alert ID for alert ping failure.
pub const ALERT_HANDLER_PARAM_LOCAL_ALERT_ID_ALERT_PINGFAIL: u32 = 0;
/// Local alert ID for escalation ping failure.
pub const ALERT_HANDLER_PARAM_LOCAL_ALERT_ID_ESC_PINGFAIL: u32 = 1;
/// Local alert ID for alert integrity failure.
pub const ALERT_HANDLER_PARAM_LOCAL_ALERT_ID_ALERT_INTEGFAIL: u32 = 2;
/// Local alert ID for escalation integrity failure.
pub const ALERT_HANDLER_PARAM_LOCAL_ALERT_ID_ESC_INTEGFAIL: u32 = 3;
/// Local alert ID for bus integrity failure.
pub const ALERT_HANDLER_PARAM_LOCAL_ALERT_ID_BUS_INTEGFAIL: u32 = 4;
/// Local alert ID for shadow register update error.
pub const ALERT_HANDLER_PARAM_LOCAL_ALERT_ID_SHADOW_REG_UPDATE_ERROR: u32 = 5;
/// Local alert ID for shadow register storage error.
pub const ALERT_HANDLER_PARAM_LOCAL_ALERT_ID_SHADOW_REG_STORAGE_ERROR: u32 = 6;
/// Last local alert ID.
pub const ALERT_HANDLER_PARAM_LOCAL_ALERT_ID_LAST: u32 = 6;
/// Register width
pub const ALERT_HANDLER_PARAM_REG_WIDTH: u32 = 32;

register_structs! {
    pub AlertHandlerRegisters {
        /// Interrupt State Register
        (0x0000 => pub(crate) intr_state: ReadWrite<u32, INTR::Register>),
        /// Interrupt Enable Register
        (0x0004 => pub(crate) intr_enable: ReadWrite<u32, INTR::Register>),
        /// Interrupt Test Register
        (0x0008 => pub(crate) intr_test: ReadWrite<u32, INTR::Register>),
        /// Register write enable for !!PING_TIMEOUT_CYC_SHADOWED and !!PING_TIMER_EN_SHADOWED.
        (0x000c => pub(crate) ping_timer_regwen: ReadWrite<u32, PING_TIMER_REGWEN::Register>),
        /// Ping timeout cycle count.
        (0x0010 => pub(crate) ping_timeout_cyc_shadowed: ReadWrite<u32, PING_TIMEOUT_CYC_SHADOWED::Register>),
        /// Ping timer enable.
        (0x0014 => pub(crate) ping_timer_en_shadowed: ReadWrite<u32, PING_TIMER_EN_SHADOWED::Register>),
        /// Register write enable for alert enable bits.
        (0x0018 => pub(crate) alert_regwen: [ReadWrite<u32, ALERT_REGWEN::Register>; 65]),
        /// Enable register for alerts.
        (0x011c => pub(crate) alert_en_shadowed: [ReadWrite<u32, ALERT_EN_SHADOWED::Register>; 65]),
        /// Class assignment of alerts.
        (0x0220 => pub(crate) alert_class_shadowed: [ReadWrite<u32, ALERT_CLASS_SHADOWED::Register>; 65]),
        /// Alert Cause Register
        (0x0324 => pub(crate) alert_cause: [ReadWrite<u32, ALERT_CAUSE::Register>; 65]),
        /// Register write enable for alert enable bits.
        (0x0428 => pub(crate) loc_alert_regwen: [ReadWrite<u32, LOC_ALERT_REGWEN::Register>; 7]),
        /// Enable register for the local alerts
        (0x0444 => pub(crate) loc_alert_en_shadowed: [ReadWrite<u32, LOC_ALERT_EN_SHADOWED::Register>; 7]),
        /// Class assignment of the local alerts
        (0x0460 => pub(crate) loc_alert_class_shadowed: [ReadWrite<u32, LOC_ALERT_CLASS_SHADOWED::Register>; 7]),
        /// Alert Cause Register for the local alerts
        (0x047c => pub(crate) loc_alert_cause: [ReadWrite<u32, LOC_ALERT_CAUSE::Register>; 7]),
        /// Lock bit for Class A configuration.
        (0x0498 => pub(crate) classa_regwen: ReadWrite<u32, CLASSA_REGWEN::Register>),
        /// Escalation control register for alert Class A. Can not be modified if !!CLASSA_REGWEN is
        /// false.
        (0x049c => pub(crate) classa_ctrl_shadowed: ReadWrite<u32, CLASSA_CTRL_SHADOWED::Register>),
        /// Clear enable for escalation protocol of Class A alerts.
        (0x04a0 => pub(crate) classa_clr_regwen: ReadWrite<u32, CLASSA_CLR_REGWEN::Register>),
        /// Clear for escalation protocol of Class A.
        (0x04a4 => pub(crate) classa_clr_shadowed: ReadWrite<u32, CLASSA_CLR_SHADOWED::Register>),
        /// Current accumulation value for alert Class A. Software can clear this register
        (0x04a8 => pub(crate) classa_accum_cnt: ReadWrite<u32, CLASSA_ACCUM_CNT::Register>),
        /// Accumulation threshold value for alert Class A.
        (0x04ac => pub(crate) classa_accum_thresh_shadowed: ReadWrite<u32, CLASSA_ACCUM_THRESH_SHADOWED::Register>),
        /// Interrupt timeout in cycles.
        (0x04b0 => pub(crate) classa_timeout_cyc_shadowed: ReadWrite<u32, CLASSA_TIMEOUT_CYC_SHADOWED::Register>),
        /// Crashdump trigger configuration for Class A.
        (0x04b4 => pub(crate) classa_crashdump_trigger_shadowed: ReadWrite<u32, CLASSA_CRASHDUMP_TRIGGER_SHADOWED::Register>),
        /// Duration of escalation phase 0 for Class A.
        (0x04b8 => pub(crate) classa_phase0_cyc_shadowed: ReadWrite<u32, CLASSA_PHASE0_CYC_SHADOWED::Register>),
        /// Duration of escalation phase 1 for Class A.
        (0x04bc => pub(crate) classa_phase1_cyc_shadowed: ReadWrite<u32, CLASSA_PHASE1_CYC_SHADOWED::Register>),
        /// Duration of escalation phase 2 for Class A.
        (0x04c0 => pub(crate) classa_phase2_cyc_shadowed: ReadWrite<u32, CLASSA_PHASE2_CYC_SHADOWED::Register>),
        /// Duration of escalation phase 3 for Class A.
        (0x04c4 => pub(crate) classa_phase3_cyc_shadowed: ReadWrite<u32, CLASSA_PHASE3_CYC_SHADOWED::Register>),
        /// Escalation counter in cycles for Class A.
        (0x04c8 => pub(crate) classa_esc_cnt: ReadWrite<u32, CLASSA_ESC_CNT::Register>),
        /// Current escalation state of Class A. See also !!CLASSA_ESC_CNT.
        (0x04cc => pub(crate) classa_state: ReadWrite<u32, CLASSA_STATE::Register>),
        /// Lock bit for Class B configuration.
        (0x04d0 => pub(crate) classb_regwen: ReadWrite<u32, CLASSB_REGWEN::Register>),
        /// Escalation control register for alert Class B. Can not be modified if !!CLASSB_REGWEN is
        /// false.
        (0x04d4 => pub(crate) classb_ctrl_shadowed: ReadWrite<u32, CLASSB_CTRL_SHADOWED::Register>),
        /// Clear enable for escalation protocol of Class B alerts.
        (0x04d8 => pub(crate) classb_clr_regwen: ReadWrite<u32, CLASSB_CLR_REGWEN::Register>),
        /// Clear for escalation protocol of Class B.
        (0x04dc => pub(crate) classb_clr_shadowed: ReadWrite<u32, CLASSB_CLR_SHADOWED::Register>),
        /// Current accumulation value for alert Class B. Software can clear this register
        (0x04e0 => pub(crate) classb_accum_cnt: ReadWrite<u32, CLASSB_ACCUM_CNT::Register>),
        /// Accumulation threshold value for alert Class B.
        (0x04e4 => pub(crate) classb_accum_thresh_shadowed: ReadWrite<u32, CLASSB_ACCUM_THRESH_SHADOWED::Register>),
        /// Interrupt timeout in cycles.
        (0x04e8 => pub(crate) classb_timeout_cyc_shadowed: ReadWrite<u32, CLASSB_TIMEOUT_CYC_SHADOWED::Register>),
        /// Crashdump trigger configuration for Class B.
        (0x04ec => pub(crate) classb_crashdump_trigger_shadowed: ReadWrite<u32, CLASSB_CRASHDUMP_TRIGGER_SHADOWED::Register>),
        /// Duration of escalation phase 0 for Class B.
        (0x04f0 => pub(crate) classb_phase0_cyc_shadowed: ReadWrite<u32, CLASSB_PHASE0_CYC_SHADOWED::Register>),
        /// Duration of escalation phase 1 for Class B.
        (0x04f4 => pub(crate) classb_phase1_cyc_shadowed: ReadWrite<u32, CLASSB_PHASE1_CYC_SHADOWED::Register>),
        /// Duration of escalation phase 2 for Class B.
        (0x04f8 => pub(crate) classb_phase2_cyc_shadowed: ReadWrite<u32, CLASSB_PHASE2_CYC_SHADOWED::Register>),
        /// Duration of escalation phase 3 for Class B.
        (0x04fc => pub(crate) classb_phase3_cyc_shadowed: ReadWrite<u32, CLASSB_PHASE3_CYC_SHADOWED::Register>),
        /// Escalation counter in cycles for Class B.
        (0x0500 => pub(crate) classb_esc_cnt: ReadWrite<u32, CLASSB_ESC_CNT::Register>),
        /// Current escalation state of Class B. See also !!CLASSB_ESC_CNT.
        (0x0504 => pub(crate) classb_state: ReadWrite<u32, CLASSB_STATE::Register>),
        /// Lock bit for Class C configuration.
        (0x0508 => pub(crate) classc_regwen: ReadWrite<u32, CLASSC_REGWEN::Register>),
        /// Escalation control register for alert Class C. Can not be modified if !!CLASSC_REGWEN is
        /// false.
        (0x050c => pub(crate) classc_ctrl_shadowed: ReadWrite<u32, CLASSC_CTRL_SHADOWED::Register>),
        /// Clear enable for escalation protocol of Class C alerts.
        (0x0510 => pub(crate) classc_clr_regwen: ReadWrite<u32, CLASSC_CLR_REGWEN::Register>),
        /// Clear for escalation protocol of Class C.
        (0x0514 => pub(crate) classc_clr_shadowed: ReadWrite<u32, CLASSC_CLR_SHADOWED::Register>),
        /// Current accumulation value for alert Class C. Software can clear this register
        (0x0518 => pub(crate) classc_accum_cnt: ReadWrite<u32, CLASSC_ACCUM_CNT::Register>),
        /// Accumulation threshold value for alert Class C.
        (0x051c => pub(crate) classc_accum_thresh_shadowed: ReadWrite<u32, CLASSC_ACCUM_THRESH_SHADOWED::Register>),
        /// Interrupt timeout in cycles.
        (0x0520 => pub(crate) classc_timeout_cyc_shadowed: ReadWrite<u32, CLASSC_TIMEOUT_CYC_SHADOWED::Register>),
        /// Crashdump trigger configuration for Class C.
        (0x0524 => pub(crate) classc_crashdump_trigger_shadowed: ReadWrite<u32, CLASSC_CRASHDUMP_TRIGGER_SHADOWED::Register>),
        /// Duration of escalation phase 0 for Class C.
        (0x0528 => pub(crate) classc_phase0_cyc_shadowed: ReadWrite<u32, CLASSC_PHASE0_CYC_SHADOWED::Register>),
        /// Duration of escalation phase 1 for Class C.
        (0x052c => pub(crate) classc_phase1_cyc_shadowed: ReadWrite<u32, CLASSC_PHASE1_CYC_SHADOWED::Register>),
        /// Duration of escalation phase 2 for Class C.
        (0x0530 => pub(crate) classc_phase2_cyc_shadowed: ReadWrite<u32, CLASSC_PHASE2_CYC_SHADOWED::Register>),
        /// Duration of escalation phase 3 for Class C.
        (0x0534 => pub(crate) classc_phase3_cyc_shadowed: ReadWrite<u32, CLASSC_PHASE3_CYC_SHADOWED::Register>),
        /// Escalation counter in cycles for Class C.
        (0x0538 => pub(crate) classc_esc_cnt: ReadWrite<u32, CLASSC_ESC_CNT::Register>),
        /// Current escalation state of Class C. See also !!CLASSC_ESC_CNT.
        (0x053c => pub(crate) classc_state: ReadWrite<u32, CLASSC_STATE::Register>),
        /// Lock bit for Class D configuration.
        (0x0540 => pub(crate) classd_regwen: ReadWrite<u32, CLASSD_REGWEN::Register>),
        /// Escalation control register for alert Class D. Can not be modified if !!CLASSD_REGWEN is
        /// false.
        (0x0544 => pub(crate) classd_ctrl_shadowed: ReadWrite<u32, CLASSD_CTRL_SHADOWED::Register>),
        /// Clear enable for escalation protocol of Class D alerts.
        (0x0548 => pub(crate) classd_clr_regwen: ReadWrite<u32, CLASSD_CLR_REGWEN::Register>),
        /// Clear for escalation protocol of Class D.
        (0x054c => pub(crate) classd_clr_shadowed: ReadWrite<u32, CLASSD_CLR_SHADOWED::Register>),
        /// Current accumulation value for alert Class D. Software can clear this register
        (0x0550 => pub(crate) classd_accum_cnt: ReadWrite<u32, CLASSD_ACCUM_CNT::Register>),
        /// Accumulation threshold value for alert Class D.
        (0x0554 => pub(crate) classd_accum_thresh_shadowed: ReadWrite<u32, CLASSD_ACCUM_THRESH_SHADOWED::Register>),
        /// Interrupt timeout in cycles.
        (0x0558 => pub(crate) classd_timeout_cyc_shadowed: ReadWrite<u32, CLASSD_TIMEOUT_CYC_SHADOWED::Register>),
        /// Crashdump trigger configuration for Class D.
        (0x055c => pub(crate) classd_crashdump_trigger_shadowed: ReadWrite<u32, CLASSD_CRASHDUMP_TRIGGER_SHADOWED::Register>),
        /// Duration of escalation phase 0 for Class D.
        (0x0560 => pub(crate) classd_phase0_cyc_shadowed: ReadWrite<u32, CLASSD_PHASE0_CYC_SHADOWED::Register>),
        /// Duration of escalation phase 1 for Class D.
        (0x0564 => pub(crate) classd_phase1_cyc_shadowed: ReadWrite<u32, CLASSD_PHASE1_CYC_SHADOWED::Register>),
        /// Duration of escalation phase 2 for Class D.
        (0x0568 => pub(crate) classd_phase2_cyc_shadowed: ReadWrite<u32, CLASSD_PHASE2_CYC_SHADOWED::Register>),
        /// Duration of escalation phase 3 for Class D.
        (0x056c => pub(crate) classd_phase3_cyc_shadowed: ReadWrite<u32, CLASSD_PHASE3_CYC_SHADOWED::Register>),
        /// Escalation counter in cycles for Class D.
        (0x0570 => pub(crate) classd_esc_cnt: ReadWrite<u32, CLASSD_ESC_CNT::Register>),
        /// Current escalation state of Class D. See also !!CLASSD_ESC_CNT.
        (0x0574 => pub(crate) classd_state: ReadWrite<u32, CLASSD_STATE::Register>),
        (0x0578 => @END),
    }
}

register_bitfields![u32,
    /// Common Interrupt Offsets
    pub(crate) INTR [
        CLASSA OFFSET(0) NUMBITS(1) [],
        CLASSB OFFSET(1) NUMBITS(1) [],
        CLASSC OFFSET(2) NUMBITS(1) [],
        CLASSD OFFSET(3) NUMBITS(1) [],
    ],
    pub(crate) PING_TIMER_REGWEN [
        PING_TIMER_REGWEN OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) PING_TIMEOUT_CYC_SHADOWED [
        PING_TIMEOUT_CYC_SHADOWED OFFSET(0) NUMBITS(16) [],
    ],
    pub(crate) PING_TIMER_EN_SHADOWED [
        PING_TIMER_EN_SHADOWED OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) ALERT_REGWEN [
        EN_0 OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) ALERT_EN_SHADOWED [
        EN_A_0 OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) ALERT_CLASS_SHADOWED [
        CLASS_A_0 OFFSET(0) NUMBITS(2) [
            CLASSA = 0,
            CLASSB = 1,
            CLASSC = 2,
            CLASSD = 3,
        ],
    ],
    pub(crate) ALERT_CAUSE [
        A_0 OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) LOC_ALERT_REGWEN [
        EN_0 OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) LOC_ALERT_EN_SHADOWED [
        EN_LA_0 OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) LOC_ALERT_CLASS_SHADOWED [
        CLASS_LA_0 OFFSET(0) NUMBITS(2) [
            CLASSA = 0,
            CLASSB = 1,
            CLASSC = 2,
            CLASSD = 3,
        ],
    ],
    pub(crate) LOC_ALERT_CAUSE [
        LA_0 OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) CLASSA_REGWEN [
        CLASSA_REGWEN OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) CLASSA_CTRL_SHADOWED [
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
    pub(crate) CLASSA_CLR_REGWEN [
        CLASSA_CLR_REGWEN OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) CLASSA_CLR_SHADOWED [
        CLASSA_CLR_SHADOWED OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) CLASSA_ACCUM_CNT [
        CLASSA_ACCUM_CNT OFFSET(0) NUMBITS(16) [],
    ],
    pub(crate) CLASSA_ACCUM_THRESH_SHADOWED [
        CLASSA_ACCUM_THRESH_SHADOWED OFFSET(0) NUMBITS(16) [],
    ],
    pub(crate) CLASSA_TIMEOUT_CYC_SHADOWED [
        CLASSA_TIMEOUT_CYC_SHADOWED OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) CLASSA_CRASHDUMP_TRIGGER_SHADOWED [
        CLASSA_CRASHDUMP_TRIGGER_SHADOWED OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) CLASSA_PHASE0_CYC_SHADOWED [
        CLASSA_PHASE0_CYC_SHADOWED OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) CLASSA_PHASE1_CYC_SHADOWED [
        CLASSA_PHASE1_CYC_SHADOWED OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) CLASSA_PHASE2_CYC_SHADOWED [
        CLASSA_PHASE2_CYC_SHADOWED OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) CLASSA_PHASE3_CYC_SHADOWED [
        CLASSA_PHASE3_CYC_SHADOWED OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) CLASSA_ESC_CNT [
        CLASSA_ESC_CNT OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) CLASSA_STATE [
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
    pub(crate) CLASSB_REGWEN [
        CLASSB_REGWEN OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) CLASSB_CTRL_SHADOWED [
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
    pub(crate) CLASSB_CLR_REGWEN [
        CLASSB_CLR_REGWEN OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) CLASSB_CLR_SHADOWED [
        CLASSB_CLR_SHADOWED OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) CLASSB_ACCUM_CNT [
        CLASSB_ACCUM_CNT OFFSET(0) NUMBITS(16) [],
    ],
    pub(crate) CLASSB_ACCUM_THRESH_SHADOWED [
        CLASSB_ACCUM_THRESH_SHADOWED OFFSET(0) NUMBITS(16) [],
    ],
    pub(crate) CLASSB_TIMEOUT_CYC_SHADOWED [
        CLASSB_TIMEOUT_CYC_SHADOWED OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) CLASSB_CRASHDUMP_TRIGGER_SHADOWED [
        CLASSB_CRASHDUMP_TRIGGER_SHADOWED OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) CLASSB_PHASE0_CYC_SHADOWED [
        CLASSB_PHASE0_CYC_SHADOWED OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) CLASSB_PHASE1_CYC_SHADOWED [
        CLASSB_PHASE1_CYC_SHADOWED OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) CLASSB_PHASE2_CYC_SHADOWED [
        CLASSB_PHASE2_CYC_SHADOWED OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) CLASSB_PHASE3_CYC_SHADOWED [
        CLASSB_PHASE3_CYC_SHADOWED OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) CLASSB_ESC_CNT [
        CLASSB_ESC_CNT OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) CLASSB_STATE [
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
    pub(crate) CLASSC_REGWEN [
        CLASSC_REGWEN OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) CLASSC_CTRL_SHADOWED [
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
    pub(crate) CLASSC_CLR_REGWEN [
        CLASSC_CLR_REGWEN OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) CLASSC_CLR_SHADOWED [
        CLASSC_CLR_SHADOWED OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) CLASSC_ACCUM_CNT [
        CLASSC_ACCUM_CNT OFFSET(0) NUMBITS(16) [],
    ],
    pub(crate) CLASSC_ACCUM_THRESH_SHADOWED [
        CLASSC_ACCUM_THRESH_SHADOWED OFFSET(0) NUMBITS(16) [],
    ],
    pub(crate) CLASSC_TIMEOUT_CYC_SHADOWED [
        CLASSC_TIMEOUT_CYC_SHADOWED OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) CLASSC_CRASHDUMP_TRIGGER_SHADOWED [
        CLASSC_CRASHDUMP_TRIGGER_SHADOWED OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) CLASSC_PHASE0_CYC_SHADOWED [
        CLASSC_PHASE0_CYC_SHADOWED OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) CLASSC_PHASE1_CYC_SHADOWED [
        CLASSC_PHASE1_CYC_SHADOWED OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) CLASSC_PHASE2_CYC_SHADOWED [
        CLASSC_PHASE2_CYC_SHADOWED OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) CLASSC_PHASE3_CYC_SHADOWED [
        CLASSC_PHASE3_CYC_SHADOWED OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) CLASSC_ESC_CNT [
        CLASSC_ESC_CNT OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) CLASSC_STATE [
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
    pub(crate) CLASSD_REGWEN [
        CLASSD_REGWEN OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) CLASSD_CTRL_SHADOWED [
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
    pub(crate) CLASSD_CLR_REGWEN [
        CLASSD_CLR_REGWEN OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) CLASSD_CLR_SHADOWED [
        CLASSD_CLR_SHADOWED OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) CLASSD_ACCUM_CNT [
        CLASSD_ACCUM_CNT OFFSET(0) NUMBITS(16) [],
    ],
    pub(crate) CLASSD_ACCUM_THRESH_SHADOWED [
        CLASSD_ACCUM_THRESH_SHADOWED OFFSET(0) NUMBITS(16) [],
    ],
    pub(crate) CLASSD_TIMEOUT_CYC_SHADOWED [
        CLASSD_TIMEOUT_CYC_SHADOWED OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) CLASSD_CRASHDUMP_TRIGGER_SHADOWED [
        CLASSD_CRASHDUMP_TRIGGER_SHADOWED OFFSET(0) NUMBITS(2) [],
    ],
    pub(crate) CLASSD_PHASE0_CYC_SHADOWED [
        CLASSD_PHASE0_CYC_SHADOWED OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) CLASSD_PHASE1_CYC_SHADOWED [
        CLASSD_PHASE1_CYC_SHADOWED OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) CLASSD_PHASE2_CYC_SHADOWED [
        CLASSD_PHASE2_CYC_SHADOWED OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) CLASSD_PHASE3_CYC_SHADOWED [
        CLASSD_PHASE3_CYC_SHADOWED OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) CLASSD_ESC_CNT [
        CLASSD_ESC_CNT OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) CLASSD_STATE [
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

// End generated register constants for alert_handler
