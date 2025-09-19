// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! PCI capabilities list
//!
//! This module provides functionality for traversing and interacting with
//! the capability list of a PCI device, as described in section 6.7 of the
//! PCI Local Bus specification.

use core::ops::Deref;

use crate::device::Device;

/// A single capability within a specific device's configuration space.
///
/// This is a generic representation of a capability. It provides only
/// low-level methods for reading and writing the capability fields. Other
/// structs in this module will wrap this struct and provide higher level
/// methods for accessing specific fields.
///
/// The `'a` lifetime references the [`Device`] this capability belongs to.
#[derive(Copy, Clone, Debug)]
pub struct BaseCap<'a> {
    dev: &'a Device,
    ptr: u16,
}

impl BaseCap<'_> {
    /// Returns the ID of this capability.
    pub fn id(&self) -> u8 {
        self.dev.read8(self.ptr)
    }

    /// Returns the next capability pointer.
    pub fn next(&self) -> u8 {
        self.dev.read8(self.ptr + 1)
    }

    /// Reads an 8-bit value relative to the base of this capability.
    #[inline]
    pub fn read8(&self, offset: u16) -> u8 {
        self.dev.read8(self.ptr + offset)
    }

    /// Writes an 8-bit value relative to the base of this capability.
    #[inline]
    pub fn write8(&self, offset: u16, val: u8) {
        self.dev.write8(self.ptr + offset, val)
    }

    /// Reads a 16-bit value relative to the base of this capability.
    #[inline]
    pub fn read16(&self, offset: u16) -> u16 {
        self.dev.read16(self.ptr + offset)
    }

    /// Writes a 16-bit value relative to the base of this capability.
    #[inline]
    pub fn write16(&self, offset: u16, val: u16) {
        self.dev.write16(self.ptr + offset, val)
    }

    /// Reads a 32-bit value relative to the base of this capability.
    #[inline]
    pub fn read32(&self, offset: u16) -> u32 {
        self.dev.read32(self.ptr + offset)
    }

    /// Writes a 32-bit value relative to the base of this capability.
    #[inline]
    pub fn write32(&self, offset: u16, val: u32) {
        self.dev.write32(self.ptr + offset, val)
    }
}

/// Message signaled interrupt (MSI) capability
pub struct MsiCap<'a>(BaseCap<'a>);

impl MsiCap<'_> {
    /// Capability ID for MSI
    pub const ID: u8 = 0x05;

    /// Returns the MSI control register
    pub fn control(&self) -> u16 {
        self.0.dev.read16(self.0.ptr + 2)
    }

    /// Returns true if MSI is enabled
    pub fn enabled(&self) -> bool {
        self.control() & 0x0001 != 0
    }

    /// Disables MSI by clearing the enable bit
    pub fn disable(&self) {
        let mut ctrl = self.control();
        ctrl &= !0x0001;
        self.0.dev.write16(self.0.ptr + 2, ctrl);
    }
}

/// Vendor specific capability
///
/// Implements `Deref` with `Target=BaseCap`, allowing the use of
/// `BaseCap` methods to read/write raw capability data.
pub struct VendorCap<'a>(BaseCap<'a>);

impl VendorCap<'_> {
    /// Capability ID for Vendor-Specific (0x09)
    pub const ID: u8 = 0x09;

    /// Returns the length field (at cap+2)
    pub fn length(&self) -> u8 {
        self.0.dev.read8(self.0.ptr + 2)
    }
}

impl<'a> Deref for VendorCap<'a> {
    type Target = BaseCap<'a>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// MSI-X Capability
pub struct MsixCap<'a>(BaseCap<'a>);

impl MsixCap<'_> {
    /// Capability ID for MSI-X
    pub const ID: u8 = 0x11;

    /// Returns the MSI-X control register
    pub fn control(&self) -> u16 {
        self.0.dev.read16(self.0.ptr + 2)
    }

    /// Returns true if MSI-X is enabled
    pub fn enabled(&self) -> bool {
        self.control() & 0x8000 != 0
    }

    /// Disables MSI-X by clearing the enable bit
    pub fn disable(&self) {
        let mut ctrl = self.control();
        ctrl &= !0x8000;
        self.0.dev.write16(self.0.ptr + 2, ctrl);
    }
}

/// Enum representing supported capability types
pub enum Cap<'a> {
    Msi(MsiCap<'a>),
    Msix(MsixCap<'a>),
    Vendor(VendorCap<'a>),
    Other(BaseCap<'a>),
}

impl<'a> From<BaseCap<'a>> for Cap<'a> {
    fn from(base: BaseCap<'a>) -> Self {
        match base.id() {
            MsiCap::ID => Cap::Msi(MsiCap(base)),
            MsixCap::ID => Cap::Msix(MsixCap(base)),
            VendorCap::ID => Cap::Vendor(VendorCap(base)),
            _ => Cap::Other(base),
        }
    }
}

/// Iterator over supported capabilities for a given device.
pub struct CapIter<'a> {
    dev: &'a Device,
    cur: u16,
    visited: u8,
}

impl<'a> CapIter<'a> {
    /// Creates a new capability iterator for the given device.
    pub(crate) fn new(dev: &'a Device) -> Self {
        let cur = dev.cap_ptr().unwrap_or(0) as u16;
        Self {
            dev,
            cur,
            visited: 0,
        }
    }
}

impl<'a> Iterator for CapIter<'a> {
    type Item = Cap<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // Pointer value of 0 marks end of capabilities list
        if self.cur == 0 {
            return None;
        }

        // Protect against malformed capabilities list
        if self.cur >= 256 {
            return None;
        }
        if self.visited > 64 {
            return None;
        }

        let base = BaseCap {
            dev: self.dev,
            ptr: self.cur,
        };

        self.cur = base.next() as u16;
        self.visited += 1;

        Some(base.into())
    }
}
