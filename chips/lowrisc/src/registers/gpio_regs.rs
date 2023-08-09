// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright lowRISC contributors 2023.

// Generated register constants for gpio.
// Built for Earlgrey-M2.5.1-RC1-438-gacc67de99
// https://github.com/lowRISC/opentitan/tree/acc67de992ee8de5f2481b1b9580679850d8b5f5
// Tree status: clean
// Build date: 2023-08-08T00:15:38

// Original reference file: hw/ip/gpio/data/gpio.hjson
use kernel::utilities::registers::ReadWrite;
use kernel::utilities::registers::{register_bitfields, register_structs};
/// Number of alerts
pub const GPIO_PARAM_NUM_ALERTS: u32 = 1;
/// Register width
pub const GPIO_PARAM_REG_WIDTH: u32 = 32;

register_structs! {
    pub GpioRegisters {
        /// Interrupt State Register
        (0x0000 => pub(crate) intr_state: ReadWrite<u32, INTR::Register>),
        /// Interrupt Enable Register
        (0x0004 => pub(crate) intr_enable: ReadWrite<u32, INTR::Register>),
        /// Interrupt Test Register
        (0x0008 => pub(crate) intr_test: ReadWrite<u32, INTR::Register>),
        /// Alert Test Register
        (0x000c => pub(crate) alert_test: ReadWrite<u32, ALERT_TEST::Register>),
        /// GPIO Input data read value
        (0x0010 => pub(crate) data_in: ReadWrite<u32, DATA_IN::Register>),
        /// GPIO direct output data write value
        (0x0014 => pub(crate) direct_out: ReadWrite<u32, DIRECT_OUT::Register>),
        /// GPIO write data lower with mask.
        (0x0018 => pub(crate) masked_out_lower: ReadWrite<u32, MASKED_OUT_LOWER::Register>),
        /// GPIO write data upper with mask.
        (0x001c => pub(crate) masked_out_upper: ReadWrite<u32, MASKED_OUT_UPPER::Register>),
        /// GPIO Output Enable.
        (0x0020 => pub(crate) direct_oe: ReadWrite<u32, DIRECT_OE::Register>),
        /// GPIO write Output Enable lower with mask.
        (0x0024 => pub(crate) masked_oe_lower: ReadWrite<u32, MASKED_OE_LOWER::Register>),
        /// GPIO write Output Enable upper with mask.
        (0x0028 => pub(crate) masked_oe_upper: ReadWrite<u32, MASKED_OE_UPPER::Register>),
        /// GPIO interrupt enable for GPIO, rising edge.
        (0x002c => pub(crate) intr_ctrl_en_rising: ReadWrite<u32, INTR::Register>),
        /// GPIO interrupt enable for GPIO, falling edge.
        (0x0030 => pub(crate) intr_ctrl_en_falling: ReadWrite<u32, INTR::Register>),
        /// GPIO interrupt enable for GPIO, level high.
        (0x0034 => pub(crate) intr_ctrl_en_lvlhigh: ReadWrite<u32, INTR::Register>),
        /// GPIO interrupt enable for GPIO, level low.
        (0x0038 => pub(crate) intr_ctrl_en_lvllow: ReadWrite<u32, INTR::Register>),
        /// filter enable for GPIO input bits.
        (0x003c => pub(crate) ctrl_en_input_filter: ReadWrite<u32, CTRL_EN_INPUT_FILTER::Register>),
        (0x0040 => @END),
    }
}

register_bitfields![u32,
    /// Common Interrupt Offsets
    pub(crate) INTR [
        GPIO_0 OFFSET(0) NUMBITS(1) [],
        GPIO_1 OFFSET(1) NUMBITS(1) [],
        GPIO_2 OFFSET(2) NUMBITS(1) [],
        GPIO_3 OFFSET(3) NUMBITS(1) [],
        GPIO_4 OFFSET(4) NUMBITS(1) [],
        GPIO_5 OFFSET(5) NUMBITS(1) [],
        GPIO_6 OFFSET(6) NUMBITS(1) [],
        GPIO_7 OFFSET(7) NUMBITS(1) [],
        GPIO_8 OFFSET(8) NUMBITS(1) [],
        GPIO_9 OFFSET(9) NUMBITS(1) [],
        GPIO_10 OFFSET(10) NUMBITS(1) [],
        GPIO_11 OFFSET(11) NUMBITS(1) [],
        GPIO_12 OFFSET(12) NUMBITS(1) [],
        GPIO_13 OFFSET(13) NUMBITS(1) [],
        GPIO_14 OFFSET(14) NUMBITS(1) [],
        GPIO_15 OFFSET(15) NUMBITS(1) [],
        GPIO_16 OFFSET(16) NUMBITS(1) [],
        GPIO_17 OFFSET(17) NUMBITS(1) [],
        GPIO_18 OFFSET(18) NUMBITS(1) [],
        GPIO_19 OFFSET(19) NUMBITS(1) [],
        GPIO_20 OFFSET(20) NUMBITS(1) [],
        GPIO_21 OFFSET(21) NUMBITS(1) [],
        GPIO_22 OFFSET(22) NUMBITS(1) [],
        GPIO_23 OFFSET(23) NUMBITS(1) [],
        GPIO_24 OFFSET(24) NUMBITS(1) [],
        GPIO_25 OFFSET(25) NUMBITS(1) [],
        GPIO_26 OFFSET(26) NUMBITS(1) [],
        GPIO_27 OFFSET(27) NUMBITS(1) [],
        GPIO_28 OFFSET(28) NUMBITS(1) [],
        GPIO_29 OFFSET(29) NUMBITS(1) [],
        GPIO_30 OFFSET(30) NUMBITS(1) [],
        GPIO_31 OFFSET(31) NUMBITS(1) [],
    ],
    pub(crate) ALERT_TEST [
        FATAL_FAULT OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) DATA_IN [
        DATA_IN OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) DIRECT_OUT [
        DIRECT_OUT OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) MASKED_OUT_LOWER [
        DATA OFFSET(0) NUMBITS(16) [],
        MASK OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) MASKED_OUT_UPPER [
        DATA OFFSET(0) NUMBITS(16) [],
        MASK OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) DIRECT_OE [
        DIRECT_OE_0 OFFSET(0) NUMBITS(1) [],
        DIRECT_OE_1 OFFSET(1) NUMBITS(1) [],
        DIRECT_OE_2 OFFSET(2) NUMBITS(1) [],
        DIRECT_OE_3 OFFSET(3) NUMBITS(1) [],
        DIRECT_OE_4 OFFSET(4) NUMBITS(1) [],
        DIRECT_OE_5 OFFSET(5) NUMBITS(1) [],
        DIRECT_OE_6 OFFSET(6) NUMBITS(1) [],
        DIRECT_OE_7 OFFSET(7) NUMBITS(1) [],
        DIRECT_OE_8 OFFSET(8) NUMBITS(1) [],
        DIRECT_OE_9 OFFSET(9) NUMBITS(1) [],
        DIRECT_OE_10 OFFSET(10) NUMBITS(1) [],
        DIRECT_OE_11 OFFSET(11) NUMBITS(1) [],
        DIRECT_OE_12 OFFSET(12) NUMBITS(1) [],
        DIRECT_OE_13 OFFSET(13) NUMBITS(1) [],
        DIRECT_OE_14 OFFSET(14) NUMBITS(1) [],
        DIRECT_OE_15 OFFSET(15) NUMBITS(1) [],
        DIRECT_OE_16 OFFSET(16) NUMBITS(1) [],
        DIRECT_OE_17 OFFSET(17) NUMBITS(1) [],
        DIRECT_OE_18 OFFSET(18) NUMBITS(1) [],
        DIRECT_OE_19 OFFSET(19) NUMBITS(1) [],
        DIRECT_OE_20 OFFSET(20) NUMBITS(1) [],
        DIRECT_OE_21 OFFSET(21) NUMBITS(1) [],
        DIRECT_OE_22 OFFSET(22) NUMBITS(1) [],
        DIRECT_OE_23 OFFSET(23) NUMBITS(1) [],
        DIRECT_OE_24 OFFSET(24) NUMBITS(1) [],
        DIRECT_OE_25 OFFSET(25) NUMBITS(1) [],
        DIRECT_OE_26 OFFSET(26) NUMBITS(1) [],
        DIRECT_OE_27 OFFSET(27) NUMBITS(1) [],
        DIRECT_OE_28 OFFSET(28) NUMBITS(1) [],
        DIRECT_OE_29 OFFSET(29) NUMBITS(1) [],
        DIRECT_OE_30 OFFSET(30) NUMBITS(1) [],
        DIRECT_OE_31 OFFSET(31) NUMBITS(1) [],
    ],
    pub(crate) MASKED_OE_LOWER [
        DATA OFFSET(0) NUMBITS(16) [],
        MASK OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) MASKED_OE_UPPER [
        DATA OFFSET(0) NUMBITS(16) [],
        MASK OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) CTRL_EN_INPUT_FILTER [
        CTRL_EN_INPUT_FILTER OFFSET(0) NUMBITS(32) [],
    ],
];

// End generated register constants for gpio
