// Generated register struct for TRIAL1

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
    pub Trial1Registers {
        (0x0 => rwtype0: ReadWrite<u32, RWTYPE0::Register>),
        (0x4 => rwtype1: ReadWrite<u32, RWTYPE1::Register>),
        (0x8 => rwtype2: ReadWrite<u32, RWTYPE2::Register>),
        (0xc => rwtype3: ReadWrite<u32, RWTYPE3::Register>),
        (0x200 => rwtype4: ReadWrite<u32, RWTYPE4::Register>),
        (0x204 => rotype0: ReadOnly<u32, ROTYPE0::Register>),
        (0x208 => w1ctype0: ReadWrite<u32, W1CTYPE0::Register>),
        (0x20c => w1ctype1: ReadWrite<u32, W1CTYPE1::Register>),
        (0x210 => w1ctype2: ReadWrite<u32, W1CTYPE2::Register>),
        (0x214 => w1stype2: ReadWrite<u32, W1STYPE2::Register>),
        (0x218 => w0ctype2: ReadWrite<u32, W0CTYPE2::Register>),
        (0x21c => r0w1ctype2: WriteOnly<u32, R0W1CTYPE2::Register>),
        (0x220 => rctype0: ReadWrite<u32, RCTYPE0::Register>),
        (0x224 => wotype0: WriteOnly<u32, WOTYPE0::Register>),
        (0x228 => mixtype0: ReadWrite<u32, MIXTYPE0::Register>),
        (0x22c => rwtype5: ReadWrite<u32, RWTYPE5::Register>),
        (0x230 => rwtype6: ReadWrite<u32, RWTYPE6::Register>),
        (0x234 => rotype1: ReadOnly<u32, ROTYPE1::Register>),
        (0x238 => rotype2: ReadOnly<u32, ROTYPE2::Register>),
        (0x23c => rwtype7: ReadWrite<u32, RWTYPE7::Register>),
    }
}

register_bitfields![u32,
    RWTYPE0 [
        RWTYPE0 OFFSET(0) NUMBITS(32) [],
    ],
    RWTYPE1 [
        FIELD0 OFFSET(0) NUMBITS(1) [],
        FIELD1 OFFSET(1) NUMBITS(1) [],
        FIELD4 OFFSET(4) NUMBITS(1) [],
        FIELD15_8 OFFSET(8) NUMBITS(8) [],
    ],
    RWTYPE2 [
        RWTYPE2 OFFSET(0) NUMBITS(32) [],
    ],
    RWTYPE3 [
        FIELD0 OFFSET(0) NUMBITS(16) [],
        FIELD1 OFFSET(16) NUMBITS(16) [],
    ],
    RWTYPE4 [
        FIELD0 OFFSET(0) NUMBITS(16) [],
        FIELD1 OFFSET(16) NUMBITS(16) [],
    ],
    ROTYPE0 [
        ROTYPE0 OFFSET(0) NUMBITS(32) [],
    ],
    W1CTYPE0 [
        W1CTYPE0 OFFSET(0) NUMBITS(32) [],
    ],
    W1CTYPE1 [
        FIELD0 OFFSET(0) NUMBITS(16) [],
        FIELD1 OFFSET(16) NUMBITS(16) [],
    ],
    W1CTYPE2 [
        W1CTYPE2 OFFSET(0) NUMBITS(32) [],
    ],
    W1STYPE2 [
        W1STYPE2 OFFSET(0) NUMBITS(32) [],
    ],
    W0CTYPE2 [
        W0CTYPE2 OFFSET(0) NUMBITS(32) [],
    ],
    R0W1CTYPE2 [
        R0W1CTYPE2 OFFSET(0) NUMBITS(32) [],
    ],
    RCTYPE0 [
        RCTYPE0 OFFSET(0) NUMBITS(32) [],
    ],
    WOTYPE0 [
        WOTYPE0 OFFSET(0) NUMBITS(32) [],
    ],
    MIXTYPE0 [
        FIELD0 OFFSET(0) NUMBITS(4) [],
        FIELD1 OFFSET(4) NUMBITS(4) [],
        FIELD2 OFFSET(8) NUMBITS(4) [],
        FIELD3 OFFSET(12) NUMBITS(4) [],
        FIELD4 OFFSET(16) NUMBITS(4) [],
        FIELD5 OFFSET(20) NUMBITS(4) [],
        FIELD6 OFFSET(24) NUMBITS(4) [],
        FIELD7 OFFSET(28) NUMBITS(4) [],
    ],
    RWTYPE5 [
        RWTYPE5 OFFSET(0) NUMBITS(32) [],
    ],
    RWTYPE6 [
        RWTYPE6 OFFSET(0) NUMBITS(32) [],
    ],
    ROTYPE1 [
        ROTYPE1 OFFSET(0) NUMBITS(32) [],
    ],
    ROTYPE2 [
        FIELD0 OFFSET(0) NUMBITS(8) [],
        FIELD1 OFFSET(8) NUMBITS(8) [],
        FIELD2 OFFSET(20) NUMBITS(12) [],
    ],
    RWTYPE7 [
        RWTYPE7 OFFSET(0) NUMBITS(32) [],
    ],
];

// Register width
pub const TRIAL1_PARAM_REG_WIDTH: u32 = 32;

// End generated register constants for TRIAL1

