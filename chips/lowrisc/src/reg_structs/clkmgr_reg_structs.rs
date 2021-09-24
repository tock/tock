// Generated register struct for CLKMGR

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
    pub ClkmgrRegisters {
        (0x0 => clk_enables: ReadWrite<u32, CLK_ENABLES::Register>),
        (0x4 => clk_hints: ReadWrite<u32, CLK_HINTS::Register>),
        (0x8 => clk_hints_status: ReadOnly<u32, CLK_HINTS_STATUS::Register>),
    }
}

register_bitfields![u32,
    CLK_ENABLES [
        CLK_FIXED_PERI_EN OFFSET(0) NUMBITS(1) [],
        CLK_USB_48MHZ_PERI_EN OFFSET(1) NUMBITS(1) [],
    ],
    CLK_HINTS [
        CLK_MAIN_AES_HINT OFFSET(0) NUMBITS(1) [],
        CLK_MAIN_HMAC_HINT OFFSET(1) NUMBITS(1) [],
    ],
    CLK_HINTS_STATUS [
        CLK_MAIN_AES_VAL OFFSET(0) NUMBITS(1) [],
        CLK_MAIN_HMAC_VAL OFFSET(1) NUMBITS(1) [],
    ],
];

// Number of clock groups
pub const CLKMGR_PARAM_NUM_GROUPS: u32 = 7;

// Register width
pub const CLKMGR_PARAM_REG_WIDTH: u32 = 32;

// End generated register constants for CLKMGR

