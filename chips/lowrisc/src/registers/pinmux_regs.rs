// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright lowRISC contributors 2023.

// Generated register constants for pinmux.
// Built for Earlgrey-M2.5.1-RC1-438-gacc67de99
// https://github.com/lowRISC/opentitan/tree/acc67de992ee8de5f2481b1b9580679850d8b5f5
// Tree status: clean
// Build date: 2023-08-08T00:15:38

// Original reference file: hw/ip/pinmux/data/pinmux.hjson
use kernel::utilities::registers::ReadWrite;
use kernel::utilities::registers::{register_bitfields, register_structs};
/// Pad attribute data width
pub const PINMUX_PARAM_ATTR_DW: u32 = 10;
/// Number of muxed peripheral inputs
pub const PINMUX_PARAM_N_MIO_PERIPH_IN: u32 = 33;
/// Number of muxed peripheral outputs
pub const PINMUX_PARAM_N_MIO_PERIPH_OUT: u32 = 32;
/// Number of muxed IO pads
pub const PINMUX_PARAM_N_MIO_PADS: u32 = 32;
/// Number of dedicated IO pads
pub const PINMUX_PARAM_N_DIO_PADS: u32 = 16;
/// Number of wakeup detectors
pub const PINMUX_PARAM_N_WKUP_DETECT: u32 = 8;
/// Number of wakeup counter bits
pub const PINMUX_PARAM_WKUP_CNT_WIDTH: u32 = 8;
/// Number of alerts
pub const PINMUX_PARAM_NUM_ALERTS: u32 = 1;
/// Register width
pub const PINMUX_PARAM_REG_WIDTH: u32 = 32;

register_structs! {
    pub PinmuxRegisters {
        /// Alert Test Register
        (0x0000 => pub(crate) alert_test: ReadWrite<u32, ALERT_TEST::Register>),
        /// Register write enable for MIO peripheral input selects.
        (0x0004 => pub(crate) mio_periph_insel_regwen: [ReadWrite<u32, MIO_PERIPH_INSEL_REGWEN::Register>; 33]),
        /// For each peripheral input, this selects the muxable pad input.
        (0x0088 => pub(crate) mio_periph_insel: [ReadWrite<u32, MIO_PERIPH_INSEL::Register>; 33]),
        /// Register write enable for MIO output selects.
        (0x010c => pub(crate) mio_outsel_regwen: [ReadWrite<u32, MIO_OUTSEL_REGWEN::Register>; 32]),
        /// For each muxable pad, this selects the peripheral output.
        (0x018c => pub(crate) mio_outsel: [ReadWrite<u32, MIO_OUTSEL::Register>; 32]),
        /// Register write enable for MIO PAD attributes.
        (0x020c => pub(crate) mio_pad_attr_regwen: [ReadWrite<u32, MIO_PAD_ATTR_REGWEN::Register>; 32]),
        /// Muxed pad attributes.
        (0x028c => pub(crate) mio_pad_attr: [ReadWrite<u32, MIO_PAD_ATTR::Register>; 32]),
        /// Register write enable for DIO PAD attributes.
        (0x030c => pub(crate) dio_pad_attr_regwen: [ReadWrite<u32, DIO_PAD_ATTR_REGWEN::Register>; 16]),
        /// Dedicated pad attributes.
        (0x034c => pub(crate) dio_pad_attr: [ReadWrite<u32, DIO_PAD_ATTR::Register>; 16]),
        /// Register indicating whether the corresponding pad is in sleep mode.
        (0x038c => pub(crate) mio_pad_sleep_status: [ReadWrite<u32, MIO_PAD_SLEEP_STATUS::Register>; 1]),
        /// Register write enable for MIO sleep value configuration.
        (0x0390 => pub(crate) mio_pad_sleep_regwen: [ReadWrite<u32, MIO_PAD_SLEEP_REGWEN::Register>; 32]),
        /// Enables the sleep mode of the corresponding muxed pad.
        (0x0410 => pub(crate) mio_pad_sleep_en: [ReadWrite<u32, MIO_PAD_SLEEP_EN::Register>; 32]),
        /// Defines sleep behavior of the corresponding muxed pad.
        (0x0490 => pub(crate) mio_pad_sleep_mode: [ReadWrite<u32, MIO_PAD_SLEEP_MODE::Register>; 32]),
        /// Register indicating whether the corresponding pad is in sleep mode.
        (0x0510 => pub(crate) dio_pad_sleep_status: [ReadWrite<u32, DIO_PAD_SLEEP_STATUS::Register>; 1]),
        /// Register write enable for DIO sleep value configuration.
        (0x0514 => pub(crate) dio_pad_sleep_regwen: [ReadWrite<u32, DIO_PAD_SLEEP_REGWEN::Register>; 16]),
        /// Enables the sleep mode of the corresponding dedicated pad.
        (0x0554 => pub(crate) dio_pad_sleep_en: [ReadWrite<u32, DIO_PAD_SLEEP_EN::Register>; 16]),
        /// Defines sleep behavior of the corresponding dedicated pad.
        (0x0594 => pub(crate) dio_pad_sleep_mode: [ReadWrite<u32, DIO_PAD_SLEEP_MODE::Register>; 16]),
        /// Register write enable for wakeup detectors.
        (0x05d4 => pub(crate) wkup_detector_regwen: [ReadWrite<u32, WKUP_DETECTOR_REGWEN::Register>; 8]),
        /// Enables for the wakeup detectors.
        (0x05f4 => pub(crate) wkup_detector_en: [ReadWrite<u32, WKUP_DETECTOR_EN::Register>; 8]),
        /// Configuration of wakeup condition detectors.
        (0x0614 => pub(crate) wkup_detector: [ReadWrite<u32, WKUP_DETECTOR::Register>; 8]),
        /// Counter thresholds for wakeup condition detectors.
        (0x0634 => pub(crate) wkup_detector_cnt_th: [ReadWrite<u32, WKUP_DETECTOR_CNT_TH::Register>; 8]),
        /// Pad selects for pad wakeup condition detectors.
        (0x0654 => pub(crate) wkup_detector_padsel: [ReadWrite<u32, WKUP_DETECTOR_PADSEL::Register>; 8]),
        /// Cause registers for wakeup detectors.
        (0x0674 => pub(crate) wkup_cause: [ReadWrite<u32, WKUP_CAUSE::Register>; 1]),
        (0x0678 => @END),
    }
}

register_bitfields![u32,
    pub(crate) ALERT_TEST [
        FATAL_FAULT OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) MIO_PERIPH_INSEL_REGWEN [
        EN_0 OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) MIO_PERIPH_INSEL [
        IN_0 OFFSET(0) NUMBITS(6) [],
    ],
    pub(crate) MIO_OUTSEL_REGWEN [
        EN_0 OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) MIO_OUTSEL [
        OUT_0 OFFSET(0) NUMBITS(6) [],
    ],
    pub(crate) MIO_PAD_ATTR_REGWEN [
        EN_0 OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) MIO_PAD_ATTR [
        INVERT_0 OFFSET(0) NUMBITS(1) [],
        VIRTUAL_OD_EN_0 OFFSET(1) NUMBITS(1) [],
        PULL_EN_0 OFFSET(2) NUMBITS(1) [],
        PULL_SELECT_0 OFFSET(3) NUMBITS(1) [
            PULL_DOWN = 0,
            PULL_UP = 1,
        ],
        KEEPER_EN_0 OFFSET(4) NUMBITS(1) [],
        SCHMITT_EN_0 OFFSET(5) NUMBITS(1) [],
        OD_EN_0 OFFSET(6) NUMBITS(1) [],
        SLEW_RATE_0 OFFSET(16) NUMBITS(2) [],
        DRIVE_STRENGTH_0 OFFSET(20) NUMBITS(4) [],
    ],
    pub(crate) DIO_PAD_ATTR_REGWEN [
        EN_0 OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) DIO_PAD_ATTR [
        INVERT_0 OFFSET(0) NUMBITS(1) [],
        VIRTUAL_OD_EN_0 OFFSET(1) NUMBITS(1) [],
        PULL_EN_0 OFFSET(2) NUMBITS(1) [],
        PULL_SELECT_0 OFFSET(3) NUMBITS(1) [
            PULL_DOWN = 0,
            PULL_UP = 1,
        ],
        KEEPER_EN_0 OFFSET(4) NUMBITS(1) [],
        SCHMITT_EN_0 OFFSET(5) NUMBITS(1) [],
        OD_EN_0 OFFSET(6) NUMBITS(1) [],
        SLEW_RATE_0 OFFSET(16) NUMBITS(2) [],
        DRIVE_STRENGTH_0 OFFSET(20) NUMBITS(4) [],
    ],
    pub(crate) MIO_PAD_SLEEP_STATUS [
        EN_0 OFFSET(0) NUMBITS(1) [],
        EN_1 OFFSET(1) NUMBITS(1) [],
        EN_2 OFFSET(2) NUMBITS(1) [],
        EN_3 OFFSET(3) NUMBITS(1) [],
        EN_4 OFFSET(4) NUMBITS(1) [],
        EN_5 OFFSET(5) NUMBITS(1) [],
        EN_6 OFFSET(6) NUMBITS(1) [],
        EN_7 OFFSET(7) NUMBITS(1) [],
        EN_8 OFFSET(8) NUMBITS(1) [],
        EN_9 OFFSET(9) NUMBITS(1) [],
        EN_10 OFFSET(10) NUMBITS(1) [],
        EN_11 OFFSET(11) NUMBITS(1) [],
        EN_12 OFFSET(12) NUMBITS(1) [],
        EN_13 OFFSET(13) NUMBITS(1) [],
        EN_14 OFFSET(14) NUMBITS(1) [],
        EN_15 OFFSET(15) NUMBITS(1) [],
        EN_16 OFFSET(16) NUMBITS(1) [],
        EN_17 OFFSET(17) NUMBITS(1) [],
        EN_18 OFFSET(18) NUMBITS(1) [],
        EN_19 OFFSET(19) NUMBITS(1) [],
        EN_20 OFFSET(20) NUMBITS(1) [],
        EN_21 OFFSET(21) NUMBITS(1) [],
        EN_22 OFFSET(22) NUMBITS(1) [],
        EN_23 OFFSET(23) NUMBITS(1) [],
        EN_24 OFFSET(24) NUMBITS(1) [],
        EN_25 OFFSET(25) NUMBITS(1) [],
        EN_26 OFFSET(26) NUMBITS(1) [],
        EN_27 OFFSET(27) NUMBITS(1) [],
        EN_28 OFFSET(28) NUMBITS(1) [],
        EN_29 OFFSET(29) NUMBITS(1) [],
        EN_30 OFFSET(30) NUMBITS(1) [],
        EN_31 OFFSET(31) NUMBITS(1) [],
    ],
    pub(crate) MIO_PAD_SLEEP_REGWEN [
        EN_0 OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) MIO_PAD_SLEEP_EN [
        EN_0 OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) MIO_PAD_SLEEP_MODE [
        OUT_0 OFFSET(0) NUMBITS(2) [
            TIE_LOW = 0,
            TIE_HIGH = 1,
            HIGH_Z = 2,
            KEEP = 3,
        ],
    ],
    pub(crate) DIO_PAD_SLEEP_STATUS [
        EN_0 OFFSET(0) NUMBITS(1) [],
        EN_1 OFFSET(1) NUMBITS(1) [],
        EN_2 OFFSET(2) NUMBITS(1) [],
        EN_3 OFFSET(3) NUMBITS(1) [],
        EN_4 OFFSET(4) NUMBITS(1) [],
        EN_5 OFFSET(5) NUMBITS(1) [],
        EN_6 OFFSET(6) NUMBITS(1) [],
        EN_7 OFFSET(7) NUMBITS(1) [],
        EN_8 OFFSET(8) NUMBITS(1) [],
        EN_9 OFFSET(9) NUMBITS(1) [],
        EN_10 OFFSET(10) NUMBITS(1) [],
        EN_11 OFFSET(11) NUMBITS(1) [],
        EN_12 OFFSET(12) NUMBITS(1) [],
        EN_13 OFFSET(13) NUMBITS(1) [],
        EN_14 OFFSET(14) NUMBITS(1) [],
        EN_15 OFFSET(15) NUMBITS(1) [],
    ],
    pub(crate) DIO_PAD_SLEEP_REGWEN [
        EN_0 OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) DIO_PAD_SLEEP_EN [
        EN_0 OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) DIO_PAD_SLEEP_MODE [
        OUT_0 OFFSET(0) NUMBITS(2) [
            TIE_LOW = 0,
            TIE_HIGH = 1,
            HIGH_Z = 2,
            KEEP = 3,
        ],
    ],
    pub(crate) WKUP_DETECTOR_REGWEN [
        EN_0 OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) WKUP_DETECTOR_EN [
        EN_0 OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) WKUP_DETECTOR [
        MODE_0 OFFSET(0) NUMBITS(3) [
            POSEDGE = 0,
            NEGEDGE = 1,
            EDGE = 2,
            TIMEDHIGH = 3,
            TIMEDLOW = 4,
        ],
        FILTER_0 OFFSET(3) NUMBITS(1) [],
        MIODIO_0 OFFSET(4) NUMBITS(1) [],
    ],
    pub(crate) WKUP_DETECTOR_CNT_TH [
        TH_0 OFFSET(0) NUMBITS(8) [],
    ],
    pub(crate) WKUP_DETECTOR_PADSEL [
        SEL_0 OFFSET(0) NUMBITS(6) [],
    ],
    pub(crate) WKUP_CAUSE [
        CAUSE_0 OFFSET(0) NUMBITS(1) [],
        CAUSE_1 OFFSET(1) NUMBITS(1) [],
        CAUSE_2 OFFSET(2) NUMBITS(1) [],
        CAUSE_3 OFFSET(3) NUMBITS(1) [],
        CAUSE_4 OFFSET(4) NUMBITS(1) [],
        CAUSE_5 OFFSET(5) NUMBITS(1) [],
        CAUSE_6 OFFSET(6) NUMBITS(1) [],
        CAUSE_7 OFFSET(7) NUMBITS(1) [],
    ],
];

// End generated register constants for pinmux
