// Generated register struct for gpio

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
    pub GpioRegisters {
        (0x0 => intr_state: ReadWrite<u32, INTR_STATE::Register>),
        (0x4 => intr_enable: ReadWrite<u32, INTR_ENABLE::Register>),
        (0x8 => intr_test: WriteOnly<u32, INTR_TEST::Register>),
        (0xc => alert_test: WriteOnly<u32, ALERT_TEST::Register>),
        (0x10 => data_in: ReadOnly<u32, DATA_IN::Register>),
        (0x14 => direct_out: ReadWrite<u32, DIRECT_OUT::Register>),
        (0x18 => masked_out_lower: ReadWrite<u32, MASKED_OUT_LOWER::Register>),
        (0x1c => masked_out_upper: ReadWrite<u32, MASKED_OUT_UPPER::Register>),
        (0x20 => direct_oe: ReadWrite<u32, DIRECT_OE::Register>),
        (0x24 => masked_oe_lower: ReadWrite<u32, MASKED_OE_LOWER::Register>),
        (0x28 => masked_oe_upper: ReadWrite<u32, MASKED_OE_UPPER::Register>),
        (0x2c => intr_ctrl_en_rising: ReadWrite<u32, INTR_CTRL_EN_RISING::Register>),
        (0x30 => intr_ctrl_en_falling: ReadWrite<u32, INTR_CTRL_EN_FALLING::Register>),
        (0x34 => intr_ctrl_en_lvlhigh: ReadWrite<u32, INTR_CTRL_EN_LVLHIGH::Register>),
        (0x38 => intr_ctrl_en_lvllow: ReadWrite<u32, INTR_CTRL_EN_LVLLOW::Register>),
        (0x3c => ctrl_en_input_filter: ReadWrite<u32, CTRL_EN_INPUT_FILTER::Register>),
    }
}

register_bitfields![u32,
    INTR_STATE [
        GPIO OFFSET(0) NUMBITS(32) [],
    ],
    INTR_ENABLE [
        GPIO OFFSET(0) NUMBITS(32) [],
    ],
    INTR_TEST [
        GPIO OFFSET(0) NUMBITS(32) [],
    ],
    ALERT_TEST [
        FATAL_FAULT OFFSET(0) NUMBITS(1) [],
    ],
    DATA_IN [
        DATA_IN OFFSET(0) NUMBITS(32) [],
    ],
    DIRECT_OUT [
        DIRECT_OUT OFFSET(0) NUMBITS(32) [],
    ],
    MASKED_OUT_LOWER [
        DATA OFFSET(0) NUMBITS(16) [],
        MASK OFFSET(16) NUMBITS(16) [],
    ],
    MASKED_OUT_UPPER [
        DATA OFFSET(0) NUMBITS(16) [],
        MASK OFFSET(16) NUMBITS(16) [],
    ],
    DIRECT_OE [
        DIRECT_OE OFFSET(0) NUMBITS(32) [],
    ],
    MASKED_OE_LOWER [
        DATA OFFSET(0) NUMBITS(16) [],
        MASK OFFSET(16) NUMBITS(16) [],
    ],
    MASKED_OE_UPPER [
        DATA OFFSET(0) NUMBITS(16) [],
        MASK OFFSET(16) NUMBITS(16) [],
    ],
    INTR_CTRL_EN_RISING [
        INTR_CTRL_EN_RISING OFFSET(0) NUMBITS(32) [],
    ],
    INTR_CTRL_EN_FALLING [
        INTR_CTRL_EN_FALLING OFFSET(0) NUMBITS(32) [],
    ],
    INTR_CTRL_EN_LVLHIGH [
        INTR_CTRL_EN_LVLHIGH OFFSET(0) NUMBITS(32) [],
    ],
    INTR_CTRL_EN_LVLLOW [
        INTR_CTRL_EN_LVLLOW OFFSET(0) NUMBITS(32) [],
    ],
    CTRL_EN_INPUT_FILTER [
        CTRL_EN_INPUT_FILTER OFFSET(0) NUMBITS(32) [],
    ],
];

// Number of alerts
pub const GPIO_PARAM_NUM_ALERTS: u32 = 1;

// Register width
pub const GPIO_PARAM_REG_WIDTH: u32 = 32;

// End generated register constants for gpio

