// Generated register struct for pattgen

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
    pub PattgenRegisters {
        (0x0 => intr_state: ReadWrite<u32, INTR_STATE::Register>),
        (0x4 => intr_enable: ReadWrite<u32, INTR_ENABLE::Register>),
        (0x8 => intr_test: WriteOnly<u32, INTR_TEST::Register>),
        (0xc => alert_test: WriteOnly<u32, ALERT_TEST::Register>),
        (0x10 => ctrl: ReadWrite<u32, CTRL::Register>),
        (0x14 => prediv_ch0: ReadWrite<u32, PREDIV_CH0::Register>),
        (0x18 => prediv_ch1: ReadWrite<u32, PREDIV_CH1::Register>),
        (0x1c => data_ch0_0: ReadWrite<u32, DATA_CH0_0::Register>),
        (0x20 => data_ch0_1: ReadWrite<u32, DATA_CH0_1::Register>),
        (0x24 => data_ch1_0: ReadWrite<u32, DATA_CH1_0::Register>),
        (0x28 => data_ch1_1: ReadWrite<u32, DATA_CH1_1::Register>),
        (0x2c => size: ReadWrite<u32, SIZE::Register>),
    }
}

register_bitfields![u32,
    INTR_STATE [
        DONE_CH0 OFFSET(0) NUMBITS(1) [],
        DONE_CH1 OFFSET(1) NUMBITS(1) [],
    ],
    INTR_ENABLE [
        DONE_CH0 OFFSET(0) NUMBITS(1) [],
        DONE_CH1 OFFSET(1) NUMBITS(1) [],
    ],
    INTR_TEST [
        DONE_CH0 OFFSET(0) NUMBITS(1) [],
        DONE_CH1 OFFSET(1) NUMBITS(1) [],
    ],
    ALERT_TEST [
        FATAL_FAULT OFFSET(0) NUMBITS(1) [],
    ],
    CTRL [
        ENABLE_CH0 OFFSET(0) NUMBITS(1) [],
        ENABLE_CH1 OFFSET(1) NUMBITS(1) [],
        POLARITY_CH0 OFFSET(2) NUMBITS(1) [],
        POLARITY_CH1 OFFSET(3) NUMBITS(1) [],
    ],
    PREDIV_CH0 [
        CLK_RATIO OFFSET(0) NUMBITS(32) [],
    ],
    PREDIV_CH1 [
        CLK_RATIO OFFSET(0) NUMBITS(32) [],
    ],
    DATA_CH0_0 [
        DATA_0 OFFSET(0) NUMBITS(32) [],
    ],
    DATA_CH0_1 [
        DATA_1 OFFSET(0) NUMBITS(32) [],
    ],
    DATA_CH1_0 [
        DATA_0 OFFSET(0) NUMBITS(32) [],
    ],
    DATA_CH1_1 [
        DATA_1 OFFSET(0) NUMBITS(32) [],
    ],
    SIZE [
        LEN_CH0 OFFSET(0) NUMBITS(6) [],
        REPS_CH0 OFFSET(6) NUMBITS(10) [],
        LEN_CH1 OFFSET(16) NUMBITS(6) [],
        REPS_CH1 OFFSET(22) NUMBITS(10) [],
    ],
];

// Number of data registers per each channel
pub const PATTGEN_PARAM_NUM_REGS_DATA: u32 = 2;

// Number of alerts
pub const PATTGEN_PARAM_NUM_ALERTS: u32 = 1;

// Register width
pub const PATTGEN_PARAM_REG_WIDTH: u32 = 32;

// PATTGEN seed pattern multi-registers for Channel 0. (common parameters)
pub const PATTGEN_DATA_CH0_DATA_FIELD_WIDTH: u32 = 32;
pub const PATTGEN_DATA_CH0_DATA_FIELDS_PER_REG: u32 = 1;
pub const PATTGEN_DATA_CH0_MULTIREG_COUNT: u32 = 2;

// PATTGEN seed pattern multi-registers for Channel 1. (common parameters)
pub const PATTGEN_DATA_CH1_DATA_FIELD_WIDTH: u32 = 32;
pub const PATTGEN_DATA_CH1_DATA_FIELDS_PER_REG: u32 = 1;
pub const PATTGEN_DATA_CH1_MULTIREG_COUNT: u32 = 2;

// End generated register constants for pattgen

