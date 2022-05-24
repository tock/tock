// Generated register constants for gpio.
// This file is licensed under either of:
//   Apache License, Version 2.0 (LICENSE-APACHE <http://www.apache.org/licenses/LICENSE-2.0>)
//   MIT License (LICENSE-MIT <http://opensource.org/licenses/MIT>)

// Built for earlgrey_silver_release_v5-5654-g222658011
// https://github.com/lowRISC/opentitan/tree/222658011c27d6c1f22f02c7f589043f207ff574
// Tree status: clean
// Build date: 2022-06-02T20:40:57

// Original reference file: hw/ip/gpio/data/gpio.hjson
// Copyright information found in the reference file:
//   Copyright lowRISC contributors.
// Licensing information found in the reference file:
//   Licensed under the Apache License, Version 2.0, see LICENSE for details.
//   SPDX-License-Identifier: Apache-2.0

use kernel::utilities::registers::ReadWrite;
use kernel::utilities::registers::{register_bitfields, register_structs};
// Number of alerts
pub const GPIO_PARAM_NUM_ALERTS: u32 = 1;
// Register width
pub const GPIO_PARAM_REG_WIDTH: u32 = 32;

register_structs! {
    pub GpioRegisters {
        // Interrupt State Register
        (0x0000 => pub(crate) intr_state: ReadWrite<u32, INTR::Register>),
        // Interrupt Enable Register
        (0x0004 => pub(crate) intr_enable: ReadWrite<u32, INTR::Register>),
        // Interrupt Test Register
        (0x0008 => pub(crate) intr_test: ReadWrite<u32, INTR::Register>),
        // Alert Test Register
        (0x000c => pub(crate) alert_test: ReadWrite<u32, ALERT_TEST::Register>),
        // GPIO Input data read value
        (0x0010 => pub(crate) data_in: ReadWrite<u32, DATA_IN::Register>),
        // GPIO direct output data write value
        (0x0014 => pub(crate) direct_out: ReadWrite<u32, DIRECT_OUT::Register>),
        // GPIO write data lower with mask.
        (0x0018 => pub(crate) masked_out_lower: ReadWrite<u32, MASKED_OUT_LOWER::Register>),
        // GPIO write data upper with mask.
        (0x001c => pub(crate) masked_out_upper: ReadWrite<u32, MASKED_OUT_UPPER::Register>),
        // GPIO Output Enable.
        (0x0020 => pub(crate) direct_oe: ReadWrite<u32, DIRECT_OE::Register>),
        // GPIO write Output Enable lower with mask.
        (0x0024 => pub(crate) masked_oe_lower: ReadWrite<u32, MASKED_OE_LOWER::Register>),
        // GPIO write Output Enable upper with mask.
        (0x0028 => pub(crate) masked_oe_upper: ReadWrite<u32, MASKED_OE_UPPER::Register>),
        // GPIO interrupt enable for GPIO, rising edge.
        (0x002c => pub(crate) intr_ctrl_en_rising: ReadWrite<u32, INTR::Register>),
        // GPIO interrupt enable for GPIO, falling edge.
        (0x0030 => pub(crate) intr_ctrl_en_falling: ReadWrite<u32, INTR::Register>),
        // GPIO interrupt enable for GPIO, level high.
        (0x0034 => pub(crate) intr_ctrl_en_lvlhigh: ReadWrite<u32, INTR::Register>),
        // GPIO interrupt enable for GPIO, level low.
        (0x0038 => pub(crate) intr_ctrl_en_lvllow: ReadWrite<u32, INTR::Register>),
        // filter enable for GPIO input bits.
        (0x003c => pub(crate) ctrl_en_input_filter: ReadWrite<u32, CTRL_EN_INPUT_FILTER::Register>),
        (0x0040 => @END),
    }
}

register_bitfields![u32,
    // Common Interrupt Offsets
    pub(crate) INTR [],
    pub(crate) ALERT_TEST [
        FATAL_FAULT OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) DATA_IN [],
    pub(crate) DIRECT_OUT [],
    pub(crate) MASKED_OUT_LOWER [
        DATA OFFSET(0) NUMBITS(16) [],
        MASK OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) MASKED_OUT_UPPER [
        DATA OFFSET(0) NUMBITS(16) [],
        MASK OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) DIRECT_OE [],
    pub(crate) MASKED_OE_LOWER [
        DATA OFFSET(0) NUMBITS(16) [],
        MASK OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) MASKED_OE_UPPER [
        DATA OFFSET(0) NUMBITS(16) [],
        MASK OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) CTRL_EN_INPUT_FILTER [],
];

// End generated register constants for gpio
