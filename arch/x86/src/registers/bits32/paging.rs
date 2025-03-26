// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

// This is inspired and adapted for Tock from the [x86](https://github.com/gz/rust-x86) crate.

//! Description of the data-structures for IA-32 paging mode.

use core::fmt;
use core::ops;
use kernel::utilities::registers::register_bitfields;
use tock_registers::LocalRegisterCopy;

/// A wrapper for a physical address.
#[repr(transparent)]
#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct PAddr(pub u32);

impl From<u32> for PAddr {
    fn from(num: u32) -> Self {
        PAddr(num)
    }
}

impl From<usize> for PAddr {
    fn from(num: usize) -> Self {
        PAddr(num as u32)
    }
}

impl From<i32> for PAddr {
    fn from(num: i32) -> Self {
        PAddr(num as u32)
    }
}

#[allow(clippy::from_over_into)]
impl Into<u32> for PAddr {
    fn into(self) -> u32 {
        self.0
    }
}

#[allow(clippy::from_over_into)]
impl Into<usize> for PAddr {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl ops::Rem for PAddr {
    type Output = PAddr;

    fn rem(self, rhs: PAddr) -> Self::Output {
        PAddr(self.0 % rhs.0)
    }
}

impl ops::Rem<u32> for PAddr {
    type Output = u32;

    fn rem(self, rhs: u32) -> Self::Output {
        self.0 % rhs
    }
}

impl ops::Rem<usize> for PAddr {
    type Output = u32;

    fn rem(self, rhs: usize) -> Self::Output {
        self.0 % (rhs as u32)
    }
}

impl ops::BitAnd for PAddr {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        PAddr(self.0 & rhs.0)
    }
}

impl ops::BitAnd<u32> for PAddr {
    type Output = u32;

    fn bitand(self, rhs: u32) -> Self::Output {
        Into::<u32>::into(self) & rhs
    }
}

impl ops::BitOr for PAddr {
    type Output = PAddr;

    fn bitor(self, rhs: PAddr) -> Self::Output {
        PAddr(self.0 | rhs.0)
    }
}

impl ops::BitOr<u32> for PAddr {
    type Output = u32;

    fn bitor(self, rhs: u32) -> Self::Output {
        self.0 | rhs
    }
}

impl fmt::LowerHex for PAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A PD Entry consists of an address and a bunch of flags.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct PDEntry(pub u32);

impl fmt::Debug for PDEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PDEntry {{ {:#x}, {:?} }}", self.address(), self.flags())
    }
}

register_bitfields![u32,
    /// PD configuration bits description
    pub PDFLAGS [
        /// Present; must be 1 to map a 4-MByte page.
        P OFFSET(0) NUMBITS(1),
        /// Read/write; if 0, writes may not be allowed to the 4-MByte page referenced by this entry.
        RW OFFSET(1) NUMBITS(1),
        /// User/supervisor; if 0, user-mode accesses are not allowed to the 4-MByte page referenced by this entry.
        US OFFSET(2) NUMBITS(1),
        /// Page-level write-through.
        PWT OFFSET(3) NUMBITS(1),
        /// Page-level cache disable.
        PCD OFFSET(4) NUMBITS(1),
        /// Accessed; indicates whether software has accessed the 4-MByte page referenced by this entry.
        A OFFSET(5) NUMBITS(1),
        /// Dirty; indicates whether software has written to the 4-MByte page referenced by this entry.
        D OFFSET(6) NUMBITS(1),
        /// Page size; if set this entry maps a 4-MByte page; otherwise, this entry references a page directory.
        PS OFFSET(7) NUMBITS(1),
        /// Global; if CR4.PGE = 1, determines whether the translation is global; ignored otherwise.
        G OFFSET(8) NUMBITS(1),
        /// If the PAT is supported, indirectly determines the memory type used to access the 4-MByte page referenced by this entry;
        /// otherwise, reserved (must be 0)
        PAT OFFSET(12) NUMBITS(1),
    ],
    pub PTFLAGS [
        /// Present; must be 1 to map a 4-MByte page.
        P OFFSET(0) NUMBITS(1),
        /// Read/write; if 0, writes may not be allowed to the 4-MByte page referenced by this entry.
        RW OFFSET(1) NUMBITS(1),
        /// User/supervisor; if 0, user-mode accesses are not allowed to the 4-MByte page referenced by this entry.
        US OFFSET(2) NUMBITS(1),
        /// Page-level write-through.
        PWT OFFSET(3) NUMBITS(1),
        /// Page-level cache disable.
        PCD OFFSET(4) NUMBITS(1),
        /// Accessed; indicates whether software has accessed the 4-MByte page referenced by this entry.
        A OFFSET(5) NUMBITS(1),
        /// Dirty; indicates whether software has written to the 4-MByte page referenced by this entry.
        D OFFSET(6) NUMBITS(1),
        /// If the PAT is supported, indirectly determines the memory type used to access the 4-MByte page referenced by this entry;
        /// otherwise, reserved (must be 0)
        PAT OFFSET(7) NUMBITS(1),
        /// Global; if CR4.PGE = 1, determines whether the translation is global; ignored otherwise.
        G OFFSET(8) NUMBITS(1),
    ],
];

pub type PDFlags = LocalRegisterCopy<u32, PDFLAGS::Register>;
pub type PTFlags = LocalRegisterCopy<u32, PTFLAGS::Register>;

/// Mask to find the physical address of an entry in a page-table.
const ADDRESS_MASK: u32 = !0xfff;
const ADDRESS_MASK_PSE: u32 = !0x3fffff;

/// Size of a base page (4 KiB)
pub const BASE_PAGE_SIZE: usize = 4096;

/// Page tables have 512 = 4096 / 32 entries.
pub const PAGE_SIZE_ENTRIES: usize = 1024;

/// A page directory.
pub type PD = [PDEntry; PAGE_SIZE_ENTRIES];

/// A page table.
pub type PT = [PTEntry; PAGE_SIZE_ENTRIES];

impl PDEntry {
    /// Creates a new PDEntry.
    ///
    /// # Arguments
    ///
    ///  * `pt` - The physical address of the page table.
    ///  * `flags`- Additional flags for the entry.
    ///
    /// # Implementation notes
    ///
    /// This doesn't support PSE-36 or PSE-40.
    pub fn new(pt: PAddr, flags: LocalRegisterCopy<u32, PDFLAGS::Register>) -> PDEntry {
        let mask = if flags.is_set(PDFLAGS::PS) {
            ADDRESS_MASK_PSE
        } else {
            ADDRESS_MASK
        };
        let pt_val = pt & mask;
        assert!(pt_val == pt.into());
        assert!(pt % BASE_PAGE_SIZE == 0);
        PDEntry(pt_val | flags.get())
    }

    /// Retrieves the physical address in this entry.
    pub fn address(self) -> PAddr {
        if self.flags().is_set(PDFLAGS::PS) {
            PAddr::from(self.0 & ADDRESS_MASK_PSE)
        } else {
            PAddr::from(self.0 & ADDRESS_MASK)
        }
    }

    /// Returns the flags corresponding to this entry.
    pub fn flags(self) -> LocalRegisterCopy<u32, PDFLAGS::Register> {
        LocalRegisterCopy::new(self.0)
    }
}

/// A PT Entry consists of an address and a bunch of flags.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct PTEntry(pub u32);

impl fmt::Debug for PTEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PTEntry {{ {:#x}, {:?} }}", self.address(), self.flags())
    }
}

impl PTEntry {
    /// Creates a new PTEntry.
    ///
    /// # Arguments
    ///
    ///  * `page` - The physical address of the backing 4 KiB page.
    ///  * `flags`- Additional flags for the entry.
    pub fn new(page: PAddr, flags: LocalRegisterCopy<u32, PTFLAGS::Register>) -> PTEntry {
        let page_val = page & ADDRESS_MASK;
        assert!(page_val == page.into());
        assert!(page % BASE_PAGE_SIZE == 0);
        PTEntry(page_val | flags.get())
    }

    /// Retrieves the physical address in this entry.
    pub fn address(self) -> PAddr {
        PAddr::from(self.0 & ADDRESS_MASK)
    }

    /// Returns the flags corresponding to this entry.
    pub fn flags(self) -> LocalRegisterCopy<u32, PTFLAGS::Register> {
        LocalRegisterCopy::new(self.0)
    }
}
