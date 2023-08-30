// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright lowRISC contributors 2023.

// Generated register constants for ast.
// Built for Earlgrey-M2.5.1-RC1-438-gacc67de99
// https://github.com/lowRISC/opentitan/tree/acc67de992ee8de5f2481b1b9580679850d8b5f5
// Tree status: clean
// Build date: 2023-08-08T00:15:38

// Original reference file: hw/top_earlgrey/ip/ast/data/ast.hjson
use kernel::utilities::registers::ReadWrite;
use kernel::utilities::registers::{register_bitfields, register_structs};
/// Number of registers in the Array-B
pub const AST_PARAM_NUM_REGS_B: u32 = 5;
/// Number of USB valid beacon pulses for clock to re-calibrate
pub const AST_PARAM_NUM_USB_BEACON_PULSES: u32 = 8;
/// Register width
pub const AST_PARAM_REG_WIDTH: u32 = 32;

register_structs! {
    pub AstRegisters {
        /// AST Register 0 for OTP/ROM Write Testing
        (0x0000 => pub(crate) rega0: ReadWrite<u32, REGA0::Register>),
        /// AST 1 Register for OTP/ROM Write Testing
        (0x0004 => pub(crate) rega1: ReadWrite<u32, REGA1::Register>),
        /// AST 2 Register for OTP/ROM Write Testing
        (0x0008 => pub(crate) rega2: ReadWrite<u32, REGA2::Register>),
        /// AST 3 Register for OTP/ROM Write Testing
        (0x000c => pub(crate) rega3: ReadWrite<u32, REGA3::Register>),
        /// AST 4 Register for OTP/ROM Write Testing
        (0x0010 => pub(crate) rega4: ReadWrite<u32, REGA4::Register>),
        /// AST 5 Register for OTP/ROM Write Testing
        (0x0014 => pub(crate) rega5: ReadWrite<u32, REGA5::Register>),
        /// AST 6 Register for OTP/ROM Write Testing
        (0x0018 => pub(crate) rega6: ReadWrite<u32, REGA6::Register>),
        /// AST 7 Register for OTP/ROM Write Testing
        (0x001c => pub(crate) rega7: ReadWrite<u32, REGA7::Register>),
        /// AST 8 Register for OTP/ROM Write Testing
        (0x0020 => pub(crate) rega8: ReadWrite<u32, REGA8::Register>),
        /// AST 9 Register for OTP/ROM Write Testing
        (0x0024 => pub(crate) rega9: ReadWrite<u32, REGA9::Register>),
        /// AST 10 Register for OTP/ROM Write Testing
        (0x0028 => pub(crate) rega10: ReadWrite<u32, REGA10::Register>),
        /// AST 11 Register for OTP/ROM Write Testing
        (0x002c => pub(crate) rega11: ReadWrite<u32, REGA11::Register>),
        /// AST 13 Register for OTP/ROM Write Testing
        (0x0030 => pub(crate) rega12: ReadWrite<u32, REGA12::Register>),
        /// AST 13 Register for OTP/ROM Write Testing
        (0x0034 => pub(crate) rega13: ReadWrite<u32, REGA13::Register>),
        /// AST 14 Register for OTP/ROM Write Testing
        (0x0038 => pub(crate) rega14: ReadWrite<u32, REGA14::Register>),
        /// AST 15 Register for OTP/ROM Write Testing
        (0x003c => pub(crate) rega15: ReadWrite<u32, REGA15::Register>),
        /// AST 16 Register for OTP/ROM Write Testing
        (0x0040 => pub(crate) rega16: ReadWrite<u32, REGA16::Register>),
        /// AST 17 Register for OTP/ROM Write Testing
        (0x0044 => pub(crate) rega17: ReadWrite<u32, REGA17::Register>),
        /// AST 18 Register for OTP/ROM Write Testing
        (0x0048 => pub(crate) rega18: ReadWrite<u32, REGA18::Register>),
        /// AST 19 Register for OTP/ROM Write Testing
        (0x004c => pub(crate) rega19: ReadWrite<u32, REGA19::Register>),
        /// AST 20 Register for OTP/ROM Write Testing
        (0x0050 => pub(crate) rega20: ReadWrite<u32, REGA20::Register>),
        /// AST 21 Register for OTP/ROM Write Testing
        (0x0054 => pub(crate) rega21: ReadWrite<u32, REGA21::Register>),
        /// AST 22 Register for OTP/ROM Write Testing
        (0x0058 => pub(crate) rega22: ReadWrite<u32, REGA22::Register>),
        /// AST 23 Register for OTP/ROM Write Testing
        (0x005c => pub(crate) rega23: ReadWrite<u32, REGA23::Register>),
        /// AST 24 Register for OTP/ROM Write Testing
        (0x0060 => pub(crate) rega24: ReadWrite<u32, REGA24::Register>),
        /// AST 25 Register for OTP/ROM Write Testing
        (0x0064 => pub(crate) rega25: ReadWrite<u32, REGA25::Register>),
        /// AST 26 Register for OTP/ROM Write Testing
        (0x0068 => pub(crate) rega26: ReadWrite<u32, REGA26::Register>),
        /// AST 27 Register for OTP/ROM Write Testing
        (0x006c => pub(crate) rega27: ReadWrite<u32, REGA27::Register>),
        /// AST 28 Register for OTP/ROM Write Testing
        (0x0070 => pub(crate) rega28: ReadWrite<u32, REGA28::Register>),
        /// AST 29 Register for OTP/ROM Write Testing
        (0x0074 => pub(crate) rega29: ReadWrite<u32, REGA29::Register>),
        /// AST 30 Register for OTP/ROM Write Testing
        (0x0078 => pub(crate) rega30: ReadWrite<u32, REGA30::Register>),
        /// AST 31 Register for OTP/ROM Write Testing
        (0x007c => pub(crate) rega31: ReadWrite<u32, REGA31::Register>),
        /// AST 32 Register for OTP/ROM Write Testing
        (0x0080 => pub(crate) rega32: ReadWrite<u32, REGA32::Register>),
        /// AST 33 Register for OTP/ROM Write Testing
        (0x0084 => pub(crate) rega33: ReadWrite<u32, REGA33::Register>),
        /// AST 34 Register for OTP/ROM Write Testing
        (0x0088 => pub(crate) rega34: ReadWrite<u32, REGA34::Register>),
        /// AST 35 Register for OTP/ROM Write Testing
        (0x008c => pub(crate) rega35: ReadWrite<u32, REGA35::Register>),
        /// AST 36 Register for OTP/ROM Write Testing
        (0x0090 => pub(crate) rega36: ReadWrite<u32, REGA36::Register>),
        /// AST 37 Register for OTP/ROM Write Testing
        (0x0094 => pub(crate) rega37: ReadWrite<u32, REGA37::Register>),
        /// AST Last Register for OTP/ROM Write Testing
        (0x0098 => pub(crate) regal: ReadWrite<u32, REGAL::Register>),
        (0x009c => _reserved1),
        /// AST Registers Array-B to set address space size
        (0x0200 => pub(crate) regb: [ReadWrite<u32, REGB::Register>; 5]),
        (0x0214 => @END),
    }
}

register_bitfields![u32,
    pub(crate) REGA0 [
        REG32 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) REGA1 [
        REG32 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) REGA2 [
        REG32 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) REGA3 [
        REG32 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) REGA4 [
        REG32 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) REGA5 [
        REG32 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) REGA6 [
        REG32 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) REGA7 [
        REG32 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) REGA8 [
        REG32 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) REGA9 [
        REG32 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) REGA10 [
        REG32 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) REGA11 [
        REG32 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) REGA12 [
        REG32 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) REGA13 [
        REG32 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) REGA14 [
        REG32 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) REGA15 [
        REG32 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) REGA16 [
        REG32 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) REGA17 [
        REG32 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) REGA18 [
        REG32 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) REGA19 [
        REG32 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) REGA20 [
        REG32 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) REGA21 [
        REG32 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) REGA22 [
        REG32 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) REGA23 [
        REG32 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) REGA24 [
        REG32 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) REGA25 [
        REG32 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) REGA26 [
        REG32 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) REGA27 [
        REG32 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) REGA28 [
        REG32 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) REGA29 [
        REG32 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) REGA30 [
        REG32 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) REGA31 [
        REG32 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) REGA32 [
        REG32 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) REGA33 [
        REG32 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) REGA34 [
        REG32 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) REGA35 [
        REG32 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) REGA36 [
        REG32 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) REGA37 [
        REG32 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) REGAL [
        REG32 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) REGB [
        REG32_0 OFFSET(0) NUMBITS(32) [],
    ],
];

// End generated register constants for ast
