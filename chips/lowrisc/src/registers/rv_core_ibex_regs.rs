// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright lowRISC contributors 2023.

// Generated register constants for rv_core_ibex.
// Built for Earlgrey-M2.5.1-RC1-438-gacc67de99
// https://github.com/lowRISC/opentitan/tree/acc67de992ee8de5f2481b1b9580679850d8b5f5
// Tree status: clean
// Build date: 2023-08-08T00:15:38

// Original reference file: hw/ip/rv_core_ibex/data/rv_core_ibex.hjson
use kernel::utilities::registers::ReadWrite;
use kernel::utilities::registers::{register_bitfields, register_structs};
/// Number of software triggerable alerts
pub const RV_CORE_IBEX_PARAM_NUM_SW_ALERTS: u32 = 2;
/// Number of translatable regions per ibex bus
pub const RV_CORE_IBEX_PARAM_NUM_REGIONS: u32 = 2;
/// Number of scratch words maintained.
pub const RV_CORE_IBEX_PARAM_NUM_SCRATCH_WORDS: u32 = 8;
/// Number of alerts
pub const RV_CORE_IBEX_PARAM_NUM_ALERTS: u32 = 4;
/// Register width
pub const RV_CORE_IBEX_PARAM_REG_WIDTH: u32 = 32;

register_structs! {
    pub RvCoreIbexRegisters {
        /// Alert Test Register
        (0x0000 => pub(crate) alert_test: ReadWrite<u32, ALERT_TEST::Register>),
        /// Software recoverable error
        (0x0004 => pub(crate) sw_recov_err: ReadWrite<u32, SW_RECOV_ERR::Register>),
        /// Software fatal error
        (0x0008 => pub(crate) sw_fatal_err: ReadWrite<u32, SW_FATAL_ERR::Register>),
        /// Ibus address control regwen.
        (0x000c => pub(crate) ibus_regwen: [ReadWrite<u32, IBUS_REGWEN::Register>; 2]),
        ///   Enable Ibus address matching
        (0x0014 => pub(crate) ibus_addr_en: [ReadWrite<u32, IBUS_ADDR_EN::Register>; 2]),
        ///   Matching region programming for ibus.
        (0x001c => pub(crate) ibus_addr_matching: [ReadWrite<u32, IBUS_ADDR_MATCHING::Register>; 2]),
        ///   The remap address after a match has been made.
        (0x0024 => pub(crate) ibus_remap_addr: [ReadWrite<u32, IBUS_REMAP_ADDR::Register>; 2]),
        /// Dbus address control regwen.
        (0x002c => pub(crate) dbus_regwen: [ReadWrite<u32, DBUS_REGWEN::Register>; 2]),
        ///   Enable dbus address matching
        (0x0034 => pub(crate) dbus_addr_en: [ReadWrite<u32, DBUS_ADDR_EN::Register>; 2]),
        ///   See !!IBUS_ADDR_MATCHING_0 for detailed description.
        (0x003c => pub(crate) dbus_addr_matching: [ReadWrite<u32, DBUS_ADDR_MATCHING::Register>; 2]),
        ///   See !!IBUS_REMAP_ADDR_0 for a detailed description.
        (0x0044 => pub(crate) dbus_remap_addr: [ReadWrite<u32, DBUS_REMAP_ADDR::Register>; 2]),
        /// Enable mask for NMI.
        (0x004c => pub(crate) nmi_enable: ReadWrite<u32, NMI_ENABLE::Register>),
        /// Current NMI state
        (0x0050 => pub(crate) nmi_state: ReadWrite<u32, NMI_STATE::Register>),
        /// error status
        (0x0054 => pub(crate) err_status: ReadWrite<u32, ERR_STATUS::Register>),
        /// Random data from EDN
        (0x0058 => pub(crate) rnd_data: ReadWrite<u32, RND_DATA::Register>),
        /// Status of random data in !!RND_DATA
        (0x005c => pub(crate) rnd_status: ReadWrite<u32, RND_STATUS::Register>),
        /// FPGA build timestamp info.
        (0x0060 => pub(crate) fpga_info: ReadWrite<u32, FPGA_INFO::Register>),
        (0x0064 => _reserved1),
        /// Memory area: Exposed tlul window for DV only purposes.
        (0x0080 => pub(crate) dv_sim_window: [ReadWrite<u32>; 8]),
        (0x00a0 => @END),
    }
}

register_bitfields![u32,
    pub(crate) ALERT_TEST [
        FATAL_SW_ERR OFFSET(0) NUMBITS(1) [],
        RECOV_SW_ERR OFFSET(1) NUMBITS(1) [],
        FATAL_HW_ERR OFFSET(2) NUMBITS(1) [],
        RECOV_HW_ERR OFFSET(3) NUMBITS(1) [],
    ],
    pub(crate) SW_RECOV_ERR [
        VAL OFFSET(0) NUMBITS(4) [],
    ],
    pub(crate) SW_FATAL_ERR [
        VAL OFFSET(0) NUMBITS(4) [],
    ],
    pub(crate) IBUS_REGWEN [
        EN_0 OFFSET(0) NUMBITS(1) [
            LOCKED = 0,
            ENABLED = 1,
        ],
    ],
    pub(crate) IBUS_ADDR_EN [
        EN_0 OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) IBUS_ADDR_MATCHING [
        VAL_0 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) IBUS_REMAP_ADDR [
        VAL_0 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) DBUS_REGWEN [
        EN_0 OFFSET(0) NUMBITS(1) [
            LOCKED = 0,
            ENABLED = 1,
        ],
    ],
    pub(crate) DBUS_ADDR_EN [
        EN_0 OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) DBUS_ADDR_MATCHING [
        VAL_0 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) DBUS_REMAP_ADDR [
        VAL_0 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) NMI_ENABLE [
        ALERT_EN OFFSET(0) NUMBITS(1) [],
        WDOG_EN OFFSET(1) NUMBITS(1) [],
    ],
    pub(crate) NMI_STATE [
        ALERT OFFSET(0) NUMBITS(1) [],
        WDOG OFFSET(1) NUMBITS(1) [],
    ],
    pub(crate) ERR_STATUS [
        REG_INTG_ERR OFFSET(0) NUMBITS(1) [],
        FATAL_INTG_ERR OFFSET(8) NUMBITS(1) [],
        FATAL_CORE_ERR OFFSET(9) NUMBITS(1) [],
        RECOV_CORE_ERR OFFSET(10) NUMBITS(1) [],
    ],
    pub(crate) RND_DATA [
        DATA OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) RND_STATUS [
        RND_DATA_VALID OFFSET(0) NUMBITS(1) [],
        RND_DATA_FIPS OFFSET(1) NUMBITS(1) [],
    ],
    pub(crate) FPGA_INFO [
        VAL OFFSET(0) NUMBITS(32) [],
    ],
];

// End generated register constants for rv_core_ibex
