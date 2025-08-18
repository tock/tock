// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Constants and helpers for working with the PCI configuration space.

use x86::registers::io;

use crate::bdf::Bdf;

/// PCI configuration address I/O port
const CONFIG_ADDRESS: u16 = 0x0CF8;

/// PCI configuration data I/O port
const CONFIG_DATA: u16 = 0x0CFC;

/// Reads a 32-bit value from the PCI configuration space.
#[inline]
pub fn read32(bdf: Bdf, offset: u16) -> u32 {
    unsafe {
        io::outl(CONFIG_ADDRESS, bdf.cfg_addr(offset));
        io::inl(CONFIG_DATA)
    }
}

/// Writes a 32-bit value to the PCI configuration space.
#[inline]
pub fn write32(bdf: Bdf, offset: u16, value: u32) {
    unsafe {
        io::outl(CONFIG_ADDRESS, bdf.cfg_addr(offset));
        io::outl(CONFIG_DATA, value);
    }
}

/// Reads a 16-bit value from the PCI configuration space.
#[inline]
pub fn read16(bdf: Bdf, offset: u16) -> u16 {
    let aligned = offset & !0x3;
    let shift = ((offset & 0x3) as u32) * 8;
    (read32(bdf, aligned) >> shift) as u16
}

/// Writes a 16-bit value to the PCI configuration space.
#[inline]
pub fn write16(bdf: Bdf, offset: u16, value: u16) {
    let aligned = offset & !0x3;
    let shift = ((offset & 0x3) as u32) * 8;
    let mask = !(0xFFFFu32 << shift);
    let cur = read32(bdf, aligned);
    let new = (cur & mask) | ((value as u32) << shift);
    write32(bdf, aligned, new);
}

/// Reads an 8-bit value from the PCI configuration space.
#[inline]
pub fn read8(bdf: Bdf, offset: u16) -> u8 {
    let aligned = offset & !0x3;
    let shift = ((offset & 0x3) as u32) * 8;
    (read32(bdf, aligned) >> shift) as u8
}

/// Writes an 8-bit value to the PCI configuration space.
#[inline]
pub fn write8(bdf: Bdf, offset: u16, value: u8) {
    let aligned = offset & !0x3;
    let shift = ((offset & 0x3) as u32) * 8;
    let mask = !(0xFFu32 << shift);
    let cur = read32(bdf, aligned);
    let new = (cur & mask) | ((value as u32) << shift);
    write32(bdf, aligned, new);
}

/// PCI configuration register offsets
pub mod offset {
    /// Vendor ID, 16 bits
    pub const VENDOR_ID: u16 = 0x00;

    /// Device ID, 16 bits
    pub const DEVICE_ID: u16 = 0x02;

    /// Command register, 16 bits
    pub const COMMAND: u16 = 0x04;

    /// Status register, 16 bits
    pub const STATUS: u16 = 0x06;

    /// Revision ID, 8 bits
    pub const REVISION_ID: u16 = 0x08;

    /// Programming interface, 8 bits
    pub const PROG_IF: u16 = 0x09;

    /// Subclass, 8 bits
    pub const SUBCLASS: u16 = 0x0A;

    /// Class code, 8 bits
    pub const CLASS_CODE: u16 = 0x0B;

    /// Cache line size, 8 bits
    pub const CACHE_LINE_SIZE: u16 = 0x0C;

    /// Latency timer, 8 bits
    pub const LATENCY_TIMER: u16 = 0x0D;

    /// Header type, 8 bits
    pub const HEADER_TYPE: u16 = 0x0E;

    /// Built-in self-test, 8 bits
    pub const BIST: u16 = 0x0F;

    /// Base address register 0, 32 bits
    pub const BAR0: u16 = 0x10;

    /// Base address register 1, 32 bits
    pub const BAR1: u16 = 0x10;

    /// Base address register 2, 32 bits
    pub const BAR2: u16 = 0x10;

    /// Base address register 3, 32 bits
    pub const BAR3: u16 = 0x10;

    /// Base address register 4, 32 bits
    pub const BAR4: u16 = 0x10;

    /// Base address register 5, 32 bits
    pub const BAR5: u16 = 0x10;

    /// Subsystem vendor ID, 16 bits
    pub const SUBSYSTEM_VENDOR_ID: u16 = 0x2C;

    /// Subsystem ID, 16 bits
    pub const SUBSYSTEM_ID: u16 = 0x2E;

    /// Capabilities pointer (head of capability list), 8 bits
    pub const CAP_PTR: u16 = 0x34;

    /// Interrupt line, 8 bits
    pub const INT_LINE: u16 = 0x3C;
}
