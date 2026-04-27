// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2026.

//! HASH registers.

use kernel::utilities::{
    StaticRef,
    registers::{ReadOnly, ReadWrite, WriteOnly, register_bitfields, register_structs},
};

register_structs! {
    /// Hash processor
    pub HashRegisters {
        /// control register
        (0x000 => pub(crate) cr: ReadWrite<u32, CR::Register>),
        /// data input register
        (0x004 => pub(crate) din: WriteOnly<u32>),
        /// start register
        (0x008 => pub(crate) str: ReadWrite<u32, STR::Register>),
        /// HASH aliased digest register 0
        /// 0x00C - 0x01C
        (0x00C => pub(crate) hra: [ReadOnly<u32>; 5]),
        /// interrupt enable register
        (0x020 => pub(crate) imr: ReadWrite<u32, IMR::Register>),
        /// status register
        (0x024 => pub(crate) sr: ReadWrite<u32, SR::Register>),
        (0x028 => _reserved0),
        /// context swap registers
        /// 0x0F8 - 0x1CC
        (0x0F8 => pub(crate) csr: [ReadWrite<u32>; 54]),
        (0x1D0 => _reserved1),
        /// digest register 0
        /// 0x310 - 0x32C
        (0x310 => pub(crate) hr: [ReadOnly<u32>; 8]),
        (0x330 => @END),
    }
}
register_bitfields![u32,
pub(crate) CR [
    /// Initialize message digest calculation
    INIT OFFSET(2) NUMBITS(1) [],
    /// DMA enable
    DMAE OFFSET(3) NUMBITS(1) [],
    /// Data type selection
    DATATYPE OFFSET(4) NUMBITS(2) [
        /// The data written into HASH_DIN are directly used by the HASH processing,
        /// without reordering.
        _32bitData = 0,
        /// Half-word. The data written into HASH_DIN are considered as two half-
        /// words, and are swapped before being used by the HASH processing.
        _16bitData = 1,
        /// Bytes. The data written into HASH_DIN are considered as four bytes, and
        /// are swapped before being used by the HASH processing.
        _8bitData = 2,
        /// Bit-string. The data written into HASH_DIN are considered as 32 bits (1st bit of
        /// the string at position 0), and are swapped before being used by the HASH processing (1st bit
        /// of the string at position 31).
        _1bitData = 3,
    ],
    /// Mode selection
    MODE OFFSET(6) NUMBITS(1) [],
    /// Algorithm selection
    ALGO OFFSET(17) NUMBITS(2) [
        /// SHA-1
        SHA_1 = 0,
        /// MD5
        MD5 = 1,
        /// SHA2-224
        SHA2_224 = 2,
        /// SHA2-256
        SHA2_256 = 3,
    ],
    /// Number of words already pushed
    NBW OFFSET(8) NUMBITS(4) [],
    /// DIN not empty
    DINNE OFFSET(12) NUMBITS(1) [],
    /// Multiple DMA Transfers
    MDMAT OFFSET(13) NUMBITS(1) [],
    /// Long key selection
    LKEY OFFSET(16) NUMBITS(1) []
],
pub(crate) DIN [
    /// Data input
    DATAIN OFFSET(0) NUMBITS(32) []
],
pub(crate) STR [
    /// Digest calculation
    DCAL OFFSET(8) NUMBITS(1) [],
    /// Number of valid bits in the last word of the message
    NBLW OFFSET(0) NUMBITS(5) []
],
pub(crate) HRA [
    /// H0
    H OFFSET(0) NUMBITS(32) []
],
pub(crate) HR [
    /// H0
    H OFFSET(0) NUMBITS(32) []
],
pub(crate) IMR [
    /// Digest calculation completion interrupt enable
    DCIE OFFSET(1) NUMBITS(1) [],
    /// Data input interrupt enable
    DINIE OFFSET(0) NUMBITS(1) []
],
pub(crate) SR [
    /// Busy bit
    BUSY OFFSET(3) NUMBITS(1) [],
    /// DMA Status
    DMAS OFFSET(2) NUMBITS(1) [],
    /// Digest calculation completion interrupt status
    DCIS OFFSET(1) NUMBITS(1) [],
    /// Data input interrupt status
    DINIS OFFSET(0) NUMBITS(1) [],
    /// Number of words expected
    NBWE OFFSET(16) NUMBITS(5) [],
    /// DIN not empty
    DINNE OFFSET(15) NUMBITS(1) [],
    /// Number of words already pushed
    NBWP OFFSET(9) NUMBITS(5) []
],
pub(crate) CSR [
    /// CS0
    CS OFFSET(0) NUMBITS(32) []
]
];
pub const HASH_BASE: StaticRef<HashRegisters> =
    unsafe { StaticRef::new(0x420C0400 as *const HashRegisters) };
