// Generated register struct for otbn

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
    pub OtbnRegisters {
        (0x0 => intr_state: ReadWrite<u32, INTR_STATE::Register>),
        (0x4 => intr_enable: ReadWrite<u32, INTR_ENABLE::Register>),
        (0x8 => intr_test: WriteOnly<u32, INTR_TEST::Register>),
        (0xc => alert_test: WriteOnly<u32, ALERT_TEST::Register>),
        (0x10 => cmd: WriteOnly<u32, CMD::Register>),
        (0x14 => status: ReadOnly<u32, STATUS::Register>),
        (0x18 => err_bits: ReadOnly<u32, ERR_BITS::Register>),
        (0x1c => start_addr: WriteOnly<u32, START_ADDR::Register>),
        (0x20 => fatal_alert_cause: ReadOnly<u32, FATAL_ALERT_CAUSE::Register>),
        (0x24 => insn_cnt: ReadOnly<u32, INSN_CNT::Register>),
    }
}

register_bitfields![u32,
    INTR_STATE [
        DONE OFFSET(0) NUMBITS(1) [],
    ],
    INTR_ENABLE [
        DONE OFFSET(0) NUMBITS(1) [],
    ],
    INTR_TEST [
        DONE OFFSET(0) NUMBITS(1) [],
    ],
    ALERT_TEST [
        FATAL OFFSET(0) NUMBITS(1) [],
        RECOV OFFSET(1) NUMBITS(1) [],
    ],
    CMD [
        CMD OFFSET(0) NUMBITS(8) [],
    ],
    STATUS [
        STATUS OFFSET(0) NUMBITS(8) [],
    ],
    ERR_BITS [
        BAD_DATA_ADDR OFFSET(0) NUMBITS(1) [],
        BAD_INSN_ADDR OFFSET(1) NUMBITS(1) [],
        CALL_STACK OFFSET(2) NUMBITS(1) [],
        ILLEGAL_INSN OFFSET(3) NUMBITS(1) [],
        LOOP OFFSET(4) NUMBITS(1) [],
        FATAL_IMEM OFFSET(5) NUMBITS(1) [],
        FATAL_DMEM OFFSET(6) NUMBITS(1) [],
        FATAL_REG OFFSET(7) NUMBITS(1) [],
        FATAL_ILLEGAL_BUS_ACCESS OFFSET(8) NUMBITS(1) [],
        FATAL_LIFECYCLE_ESCALATION OFFSET(9) NUMBITS(1) [],
    ],
    START_ADDR [
        START_ADDR OFFSET(0) NUMBITS(32) [],
    ],
    FATAL_ALERT_CAUSE [
        BUS_INTEGRITY_ERROR OFFSET(0) NUMBITS(1) [],
        IMEM_ERROR OFFSET(1) NUMBITS(1) [],
        DMEM_ERROR OFFSET(2) NUMBITS(1) [],
        REG_ERROR OFFSET(3) NUMBITS(1) [],
        ILLEGAL_BUS_ACCESS OFFSET(4) NUMBITS(1) [],
        LIFECYCLE_ESCALATION OFFSET(5) NUMBITS(1) [],
    ],
    INSN_CNT [
        INSN_CNT OFFSET(0) NUMBITS(32) [],
    ],
];

// Number of alerts
pub const OTBN_PARAM_NUM_ALERTS: u32 = 2;

// Register width
pub const OTBN_PARAM_REG_WIDTH: u32 = 32;

// Memory area: Instruction Memory.
pub const OTBN_IMEM_REG_OFFSET: usize = 0x4000;
pub const OTBN_IMEM_SIZE_WORDS: u32 = 1024;
pub const OTBN_IMEM_SIZE_BYTES: u32 = 4096;
// Memory area: Data Memory.
pub const OTBN_DMEM_REG_OFFSET: usize = 0x8000;
pub const OTBN_DMEM_SIZE_WORDS: u32 = 1024;
pub const OTBN_DMEM_SIZE_BYTES: u32 = 4096;
// End generated register constants for otbn

