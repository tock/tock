// Generated register struct for RV_PLIC

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
    pub Rv_PlicRegisters {
        (0x0 => prio0: ReadWrite<u32, PRIO0::Register>),
        (0x4 => prio1: ReadWrite<u32, PRIO1::Register>),
        (0x8 => prio2: ReadWrite<u32, PRIO2::Register>),
        (0xc => prio3: ReadWrite<u32, PRIO3::Register>),
        (0x10 => prio4: ReadWrite<u32, PRIO4::Register>),
        (0x14 => prio5: ReadWrite<u32, PRIO5::Register>),
        (0x18 => prio6: ReadWrite<u32, PRIO6::Register>),
        (0x1c => prio7: ReadWrite<u32, PRIO7::Register>),
        (0x20 => prio8: ReadWrite<u32, PRIO8::Register>),
        (0x24 => prio9: ReadWrite<u32, PRIO9::Register>),
        (0x28 => prio10: ReadWrite<u32, PRIO10::Register>),
        (0x2c => prio11: ReadWrite<u32, PRIO11::Register>),
        (0x30 => prio12: ReadWrite<u32, PRIO12::Register>),
        (0x34 => prio13: ReadWrite<u32, PRIO13::Register>),
        (0x38 => prio14: ReadWrite<u32, PRIO14::Register>),
        (0x3c => prio15: ReadWrite<u32, PRIO15::Register>),
        (0x40 => prio16: ReadWrite<u32, PRIO16::Register>),
        (0x44 => prio17: ReadWrite<u32, PRIO17::Register>),
        (0x48 => prio18: ReadWrite<u32, PRIO18::Register>),
        (0x4c => prio19: ReadWrite<u32, PRIO19::Register>),
        (0x50 => prio20: ReadWrite<u32, PRIO20::Register>),
        (0x54 => prio21: ReadWrite<u32, PRIO21::Register>),
        (0x58 => prio22: ReadWrite<u32, PRIO22::Register>),
        (0x5c => prio23: ReadWrite<u32, PRIO23::Register>),
        (0x60 => prio24: ReadWrite<u32, PRIO24::Register>),
        (0x64 => prio25: ReadWrite<u32, PRIO25::Register>),
        (0x68 => prio26: ReadWrite<u32, PRIO26::Register>),
        (0x6c => prio27: ReadWrite<u32, PRIO27::Register>),
        (0x70 => prio28: ReadWrite<u32, PRIO28::Register>),
        (0x74 => prio29: ReadWrite<u32, PRIO29::Register>),
        (0x78 => prio30: ReadWrite<u32, PRIO30::Register>),
        (0x7c => prio31: ReadWrite<u32, PRIO31::Register>),
        (0x1000 => ip: ReadOnly<u32, IP::Register>),
        (0x2000 => ie0: ReadWrite<u32, IE0::Register>),
        (0x200000 => threshold0: ReadWrite<u32, THRESHOLD0::Register>),
        (0x200004 => cc0: ReadWrite<u32, CC0::Register>),
        (0x4000000 => msip0: ReadWrite<u32, MSIP0::Register>),
        (0x4004000 => alert_test: WriteOnly<u32, ALERT_TEST::Register>),
    }
}

register_bitfields![u32,
    PRIO0 [
        PRIO0 OFFSET(0) NUMBITS(3) [],
    ],
    PRIO1 [
        PRIO1 OFFSET(0) NUMBITS(3) [],
    ],
    PRIO2 [
        PRIO2 OFFSET(0) NUMBITS(3) [],
    ],
    PRIO3 [
        PRIO3 OFFSET(0) NUMBITS(3) [],
    ],
    PRIO4 [
        PRIO4 OFFSET(0) NUMBITS(3) [],
    ],
    PRIO5 [
        PRIO5 OFFSET(0) NUMBITS(3) [],
    ],
    PRIO6 [
        PRIO6 OFFSET(0) NUMBITS(3) [],
    ],
    PRIO7 [
        PRIO7 OFFSET(0) NUMBITS(3) [],
    ],
    PRIO8 [
        PRIO8 OFFSET(0) NUMBITS(3) [],
    ],
    PRIO9 [
        PRIO9 OFFSET(0) NUMBITS(3) [],
    ],
    PRIO10 [
        PRIO10 OFFSET(0) NUMBITS(3) [],
    ],
    PRIO11 [
        PRIO11 OFFSET(0) NUMBITS(3) [],
    ],
    PRIO12 [
        PRIO12 OFFSET(0) NUMBITS(3) [],
    ],
    PRIO13 [
        PRIO13 OFFSET(0) NUMBITS(3) [],
    ],
    PRIO14 [
        PRIO14 OFFSET(0) NUMBITS(3) [],
    ],
    PRIO15 [
        PRIO15 OFFSET(0) NUMBITS(3) [],
    ],
    PRIO16 [
        PRIO16 OFFSET(0) NUMBITS(3) [],
    ],
    PRIO17 [
        PRIO17 OFFSET(0) NUMBITS(3) [],
    ],
    PRIO18 [
        PRIO18 OFFSET(0) NUMBITS(3) [],
    ],
    PRIO19 [
        PRIO19 OFFSET(0) NUMBITS(3) [],
    ],
    PRIO20 [
        PRIO20 OFFSET(0) NUMBITS(3) [],
    ],
    PRIO21 [
        PRIO21 OFFSET(0) NUMBITS(3) [],
    ],
    PRIO22 [
        PRIO22 OFFSET(0) NUMBITS(3) [],
    ],
    PRIO23 [
        PRIO23 OFFSET(0) NUMBITS(3) [],
    ],
    PRIO24 [
        PRIO24 OFFSET(0) NUMBITS(3) [],
    ],
    PRIO25 [
        PRIO25 OFFSET(0) NUMBITS(3) [],
    ],
    PRIO26 [
        PRIO26 OFFSET(0) NUMBITS(3) [],
    ],
    PRIO27 [
        PRIO27 OFFSET(0) NUMBITS(3) [],
    ],
    PRIO28 [
        PRIO28 OFFSET(0) NUMBITS(3) [],
    ],
    PRIO29 [
        PRIO29 OFFSET(0) NUMBITS(3) [],
    ],
    PRIO30 [
        PRIO30 OFFSET(0) NUMBITS(3) [],
    ],
    PRIO31 [
        PRIO31 OFFSET(0) NUMBITS(3) [],
    ],
    IP [
        P_0 OFFSET(0) NUMBITS(1) [],
        P_1 OFFSET(1) NUMBITS(1) [],
        P_2 OFFSET(2) NUMBITS(1) [],
        P_3 OFFSET(3) NUMBITS(1) [],
        P_4 OFFSET(4) NUMBITS(1) [],
        P_5 OFFSET(5) NUMBITS(1) [],
        P_6 OFFSET(6) NUMBITS(1) [],
        P_7 OFFSET(7) NUMBITS(1) [],
        P_8 OFFSET(8) NUMBITS(1) [],
        P_9 OFFSET(9) NUMBITS(1) [],
        P_10 OFFSET(10) NUMBITS(1) [],
        P_11 OFFSET(11) NUMBITS(1) [],
        P_12 OFFSET(12) NUMBITS(1) [],
        P_13 OFFSET(13) NUMBITS(1) [],
        P_14 OFFSET(14) NUMBITS(1) [],
        P_15 OFFSET(15) NUMBITS(1) [],
        P_16 OFFSET(16) NUMBITS(1) [],
        P_17 OFFSET(17) NUMBITS(1) [],
        P_18 OFFSET(18) NUMBITS(1) [],
        P_19 OFFSET(19) NUMBITS(1) [],
        P_20 OFFSET(20) NUMBITS(1) [],
        P_21 OFFSET(21) NUMBITS(1) [],
        P_22 OFFSET(22) NUMBITS(1) [],
        P_23 OFFSET(23) NUMBITS(1) [],
        P_24 OFFSET(24) NUMBITS(1) [],
        P_25 OFFSET(25) NUMBITS(1) [],
        P_26 OFFSET(26) NUMBITS(1) [],
        P_27 OFFSET(27) NUMBITS(1) [],
        P_28 OFFSET(28) NUMBITS(1) [],
        P_29 OFFSET(29) NUMBITS(1) [],
        P_30 OFFSET(30) NUMBITS(1) [],
        P_31 OFFSET(31) NUMBITS(1) [],
    ],
    IE0 [
        E_0 OFFSET(0) NUMBITS(1) [],
        E_1 OFFSET(1) NUMBITS(1) [],
        E_2 OFFSET(2) NUMBITS(1) [],
        E_3 OFFSET(3) NUMBITS(1) [],
        E_4 OFFSET(4) NUMBITS(1) [],
        E_5 OFFSET(5) NUMBITS(1) [],
        E_6 OFFSET(6) NUMBITS(1) [],
        E_7 OFFSET(7) NUMBITS(1) [],
        E_8 OFFSET(8) NUMBITS(1) [],
        E_9 OFFSET(9) NUMBITS(1) [],
        E_10 OFFSET(10) NUMBITS(1) [],
        E_11 OFFSET(11) NUMBITS(1) [],
        E_12 OFFSET(12) NUMBITS(1) [],
        E_13 OFFSET(13) NUMBITS(1) [],
        E_14 OFFSET(14) NUMBITS(1) [],
        E_15 OFFSET(15) NUMBITS(1) [],
        E_16 OFFSET(16) NUMBITS(1) [],
        E_17 OFFSET(17) NUMBITS(1) [],
        E_18 OFFSET(18) NUMBITS(1) [],
        E_19 OFFSET(19) NUMBITS(1) [],
        E_20 OFFSET(20) NUMBITS(1) [],
        E_21 OFFSET(21) NUMBITS(1) [],
        E_22 OFFSET(22) NUMBITS(1) [],
        E_23 OFFSET(23) NUMBITS(1) [],
        E_24 OFFSET(24) NUMBITS(1) [],
        E_25 OFFSET(25) NUMBITS(1) [],
        E_26 OFFSET(26) NUMBITS(1) [],
        E_27 OFFSET(27) NUMBITS(1) [],
        E_28 OFFSET(28) NUMBITS(1) [],
        E_29 OFFSET(29) NUMBITS(1) [],
        E_30 OFFSET(30) NUMBITS(1) [],
        E_31 OFFSET(31) NUMBITS(1) [],
    ],
    THRESHOLD0 [
        THRESHOLD0 OFFSET(0) NUMBITS(3) [],
    ],
    CC0 [
        CC0 OFFSET(0) NUMBITS(5) [],
    ],
    MSIP0 [
        MSIP0 OFFSET(0) NUMBITS(1) [],
    ],
    ALERT_TEST [
        FATAL_FAULT OFFSET(0) NUMBITS(1) [],
    ],
];

// Number of interrupt sources
pub const RV_PLIC_PARAM_NUM_SRC: u32 = 32;

// Number of Targets (Harts)
pub const RV_PLIC_PARAM_NUM_TARGET: u32 = 1;

// Width of priority signals
pub const RV_PLIC_PARAM_PRIO_WIDTH: u32 = 3;

// Number of alerts
pub const RV_PLIC_PARAM_NUM_ALERTS: u32 = 1;

// Register width
pub const RV_PLIC_PARAM_REG_WIDTH: u32 = 32;

// Interrupt Pending (common parameters)
pub const RV_PLIC_IP_P_FIELD_WIDTH: u32 = 1;
pub const RV_PLIC_IP_P_FIELDS_PER_REG: u32 = 32;
pub const RV_PLIC_IP_MULTIREG_COUNT: u32 = 1;

// Interrupt Enable for Target 0 (common parameters)
pub const RV_PLIC_IE0_E_FIELD_WIDTH: u32 = 1;
pub const RV_PLIC_IE0_E_FIELDS_PER_REG: u32 = 32;
pub const RV_PLIC_IE0_MULTIREG_COUNT: u32 = 1;

// End generated register constants for RV_PLIC

