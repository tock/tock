// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright lowRISC contributors 2023.

// Generated register constants for clkmgr.
// Built for Earlgrey-M2.5.1-RC1-438-gacc67de99
// https://github.com/lowRISC/opentitan/tree/acc67de992ee8de5f2481b1b9580679850d8b5f5
// Tree status: clean
// Build date: 2023-08-08T00:15:38

// Original reference file: hw/top_earlgrey/ip/clkmgr/data/autogen/clkmgr.hjson
use kernel::utilities::registers::ReadWrite;
use kernel::utilities::registers::{register_bitfields, register_structs};
/// Number of clock groups
pub const CLKMGR_PARAM_NUM_GROUPS: u32 = 7;
/// Number of SW gateable clocks
pub const CLKMGR_PARAM_NUM_SW_GATEABLE_CLOCKS: u32 = 4;
/// Number of hintable clocks
pub const CLKMGR_PARAM_NUM_HINTABLE_CLOCKS: u32 = 4;
/// Number of alerts
pub const CLKMGR_PARAM_NUM_ALERTS: u32 = 2;
/// Register width
pub const CLKMGR_PARAM_REG_WIDTH: u32 = 32;

register_structs! {
    pub ClkmgrRegisters {
        /// Alert Test Register
        (0x0000 => pub(crate) alert_test: ReadWrite<u32, ALERT_TEST::Register>),
        /// External clock control write enable
        (0x0004 => pub(crate) extclk_ctrl_regwen: ReadWrite<u32, EXTCLK_CTRL_REGWEN::Register>),
        /// Select external clock
        (0x0008 => pub(crate) extclk_ctrl: ReadWrite<u32, EXTCLK_CTRL::Register>),
        /// Status of requested external clock switch
        (0x000c => pub(crate) extclk_status: ReadWrite<u32, EXTCLK_STATUS::Register>),
        /// Jitter write enable
        (0x0010 => pub(crate) jitter_regwen: ReadWrite<u32, JITTER_REGWEN::Register>),
        /// Enable jittery clock
        (0x0014 => pub(crate) jitter_enable: ReadWrite<u32, JITTER_ENABLE::Register>),
        /// Clock enable for software gateable clocks.
        (0x0018 => pub(crate) clk_enables: ReadWrite<u32, CLK_ENABLES::Register>),
        /// Clock hint for software gateable transactional clocks during active mode.
        (0x001c => pub(crate) clk_hints: ReadWrite<u32, CLK_HINTS::Register>),
        /// Since the final state of !!CLK_HINTS is not always determined by software,
        (0x0020 => pub(crate) clk_hints_status: ReadWrite<u32, CLK_HINTS_STATUS::Register>),
        /// Measurement control write enable
        (0x0024 => pub(crate) measure_ctrl_regwen: ReadWrite<u32, MEASURE_CTRL_REGWEN::Register>),
        /// Enable for measurement control
        (0x0028 => pub(crate) io_meas_ctrl_en: ReadWrite<u32, IO_MEAS_CTRL_EN::Register>),
        /// Configuration controls for io measurement.
        (0x002c => pub(crate) io_meas_ctrl_shadowed: ReadWrite<u32, IO_MEAS_CTRL_SHADOWED::Register>),
        /// Enable for measurement control
        (0x0030 => pub(crate) io_div2_meas_ctrl_en: ReadWrite<u32, IO_DIV2_MEAS_CTRL_EN::Register>),
        /// Configuration controls for io_div2 measurement.
        (0x0034 => pub(crate) io_div2_meas_ctrl_shadowed: ReadWrite<u32, IO_DIV2_MEAS_CTRL_SHADOWED::Register>),
        /// Enable for measurement control
        (0x0038 => pub(crate) io_div4_meas_ctrl_en: ReadWrite<u32, IO_DIV4_MEAS_CTRL_EN::Register>),
        /// Configuration controls for io_div4 measurement.
        (0x003c => pub(crate) io_div4_meas_ctrl_shadowed: ReadWrite<u32, IO_DIV4_MEAS_CTRL_SHADOWED::Register>),
        /// Enable for measurement control
        (0x0040 => pub(crate) main_meas_ctrl_en: ReadWrite<u32, MAIN_MEAS_CTRL_EN::Register>),
        /// Configuration controls for main measurement.
        (0x0044 => pub(crate) main_meas_ctrl_shadowed: ReadWrite<u32, MAIN_MEAS_CTRL_SHADOWED::Register>),
        /// Enable for measurement control
        (0x0048 => pub(crate) usb_meas_ctrl_en: ReadWrite<u32, USB_MEAS_CTRL_EN::Register>),
        /// Configuration controls for usb measurement.
        (0x004c => pub(crate) usb_meas_ctrl_shadowed: ReadWrite<u32, USB_MEAS_CTRL_SHADOWED::Register>),
        /// Recoverable Error code
        (0x0050 => pub(crate) recov_err_code: ReadWrite<u32, RECOV_ERR_CODE::Register>),
        /// Error code
        (0x0054 => pub(crate) fatal_err_code: ReadWrite<u32, FATAL_ERR_CODE::Register>),
        (0x0058 => @END),
    }
}

register_bitfields![u32,
    pub(crate) ALERT_TEST [
        RECOV_FAULT OFFSET(0) NUMBITS(1) [],
        FATAL_FAULT OFFSET(1) NUMBITS(1) [],
    ],
    pub(crate) EXTCLK_CTRL_REGWEN [
        EN OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) EXTCLK_CTRL [
        SEL OFFSET(0) NUMBITS(4) [],
        HI_SPEED_SEL OFFSET(4) NUMBITS(4) [],
    ],
    pub(crate) EXTCLK_STATUS [
        ACK OFFSET(0) NUMBITS(4) [],
    ],
    pub(crate) JITTER_REGWEN [
        EN OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) JITTER_ENABLE [
        VAL OFFSET(0) NUMBITS(4) [],
    ],
    pub(crate) CLK_ENABLES [
        CLK_IO_DIV4_PERI_EN OFFSET(0) NUMBITS(1) [],
        CLK_IO_DIV2_PERI_EN OFFSET(1) NUMBITS(1) [],
        CLK_IO_PERI_EN OFFSET(2) NUMBITS(1) [],
        CLK_USB_PERI_EN OFFSET(3) NUMBITS(1) [],
    ],
    pub(crate) CLK_HINTS [
        CLK_MAIN_AES_HINT OFFSET(0) NUMBITS(1) [],
        CLK_MAIN_HMAC_HINT OFFSET(1) NUMBITS(1) [],
        CLK_MAIN_KMAC_HINT OFFSET(2) NUMBITS(1) [],
        CLK_MAIN_OTBN_HINT OFFSET(3) NUMBITS(1) [],
    ],
    pub(crate) CLK_HINTS_STATUS [
        CLK_MAIN_AES_VAL OFFSET(0) NUMBITS(1) [],
        CLK_MAIN_HMAC_VAL OFFSET(1) NUMBITS(1) [],
        CLK_MAIN_KMAC_VAL OFFSET(2) NUMBITS(1) [],
        CLK_MAIN_OTBN_VAL OFFSET(3) NUMBITS(1) [],
    ],
    pub(crate) MEASURE_CTRL_REGWEN [
        EN OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) IO_MEAS_CTRL_EN [
        EN OFFSET(0) NUMBITS(4) [],
    ],
    pub(crate) IO_MEAS_CTRL_SHADOWED [
        HI OFFSET(0) NUMBITS(10) [],
        LO OFFSET(10) NUMBITS(10) [],
    ],
    pub(crate) IO_DIV2_MEAS_CTRL_EN [
        EN OFFSET(0) NUMBITS(4) [],
    ],
    pub(crate) IO_DIV2_MEAS_CTRL_SHADOWED [
        HI OFFSET(0) NUMBITS(9) [],
        LO OFFSET(9) NUMBITS(9) [],
    ],
    pub(crate) IO_DIV4_MEAS_CTRL_EN [
        EN OFFSET(0) NUMBITS(4) [],
    ],
    pub(crate) IO_DIV4_MEAS_CTRL_SHADOWED [
        HI OFFSET(0) NUMBITS(8) [],
        LO OFFSET(8) NUMBITS(8) [],
    ],
    pub(crate) MAIN_MEAS_CTRL_EN [
        EN OFFSET(0) NUMBITS(4) [],
    ],
    pub(crate) MAIN_MEAS_CTRL_SHADOWED [
        HI OFFSET(0) NUMBITS(10) [],
        LO OFFSET(10) NUMBITS(10) [],
    ],
    pub(crate) USB_MEAS_CTRL_EN [
        EN OFFSET(0) NUMBITS(4) [],
    ],
    pub(crate) USB_MEAS_CTRL_SHADOWED [
        HI OFFSET(0) NUMBITS(9) [],
        LO OFFSET(9) NUMBITS(9) [],
    ],
    pub(crate) RECOV_ERR_CODE [
        SHADOW_UPDATE_ERR OFFSET(0) NUMBITS(1) [],
        IO_MEASURE_ERR OFFSET(1) NUMBITS(1) [],
        IO_DIV2_MEASURE_ERR OFFSET(2) NUMBITS(1) [],
        IO_DIV4_MEASURE_ERR OFFSET(3) NUMBITS(1) [],
        MAIN_MEASURE_ERR OFFSET(4) NUMBITS(1) [],
        USB_MEASURE_ERR OFFSET(5) NUMBITS(1) [],
        IO_TIMEOUT_ERR OFFSET(6) NUMBITS(1) [],
        IO_DIV2_TIMEOUT_ERR OFFSET(7) NUMBITS(1) [],
        IO_DIV4_TIMEOUT_ERR OFFSET(8) NUMBITS(1) [],
        MAIN_TIMEOUT_ERR OFFSET(9) NUMBITS(1) [],
        USB_TIMEOUT_ERR OFFSET(10) NUMBITS(1) [],
    ],
    pub(crate) FATAL_ERR_CODE [
        REG_INTG OFFSET(0) NUMBITS(1) [],
        IDLE_CNT OFFSET(1) NUMBITS(1) [],
        SHADOW_STORAGE_ERR OFFSET(2) NUMBITS(1) [],
    ],
];

// End generated register constants for clkmgr
