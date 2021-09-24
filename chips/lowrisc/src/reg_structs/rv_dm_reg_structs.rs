// Generated register struct for rv_dm

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
    pub Rv_DmRegisters {
        (0x0 => alert_test: WriteOnly<u32, ALERT_TEST::Register>),
    }
}

register_bitfields![u32,
    ALERT_TEST [
        FATAL_FAULT OFFSET(0) NUMBITS(1) [],
    ],
];

// Number of hardware threads in the system.
pub const RV_DM_PARAM_NR_HARTS: u32 = 1;

// Number of alerts
pub const RV_DM_PARAM_NUM_ALERTS: u32 = 1;

// Register width
pub const RV_DM_PARAM_REG_WIDTH: u32 = 32;

// Memory area: Access window into the debug ROM.
pub const RV_DM_ROM_REG_OFFSET: usize = 0x0;
pub const RV_DM_ROM_SIZE_WORDS: u32 = 1024;
pub const RV_DM_ROM_SIZE_BYTES: u32 = 4096;
// End generated register constants for rv_dm

