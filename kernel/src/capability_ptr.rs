// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Google LLC 2024.

//! Defines the CapabilityPtr type

use core::fmt::{Formatter, LowerHex, UpperHex};
use core::ops::AddAssign;

/// A pointer to userspace memory with implied authority.
///
/// A [CapabilityPtr] points to memory a userspace processes may be
/// permitted to read, write, or execute. It is sized exactly to a
/// register that can pass values between userspace and kernel and at
/// least the size of a word ([usize]) [^note1]. Operations on the
/// pointer may affect permissions, e.g. offsetting the pointer beyond
/// the bounds of the memory object invalidates it. Like a `*const
/// ()`, a [CapabilityPtr] may also "hide" information by storing a
/// word of data with no memory access permissions.
///
/// [CapabilityPtr] should be used to store or pass between the kernel
/// and userspace a value that may represent a valid userspace reference,
/// when one party intends the other to access it.
///
/// [^note1]: Depending on the architecture, the size of a
/// [CapabilityPtr] may be a word size or larger, e.g., if registers
/// can store metadata such as access permissions.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct CapabilityPtr {
    ptr: *const (),
}

impl Default for CapabilityPtr {
    fn default() -> Self {
        Self {
            ptr: core::ptr::null(),
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum MetaPermissions {
    None,
    Read,
    Write,
    ReadWrite,
    Execute,
}

impl From<CapabilityPtr> for usize {
    /// Provenance note: may not expose provenance
    #[inline]
    fn from(from: CapabilityPtr) -> Self {
        from.ptr as usize
    }
}

impl From<usize> for CapabilityPtr {
    /// Provenance note: may have null provenance
    #[inline]
    fn from(from: usize) -> Self {
        Self {
            ptr: from as *const (),
        }
    }
}

impl UpperHex for CapabilityPtr {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        UpperHex::fmt(&(self.ptr as usize), f)
    }
}

impl LowerHex for CapabilityPtr {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        LowerHex::fmt(&(self.ptr as usize), f)
    }
}

impl AddAssign<usize> for CapabilityPtr {
    #[inline]
    fn add_assign(&mut self, rhs: usize) {
        self.ptr = (self.ptr as *const u8).wrapping_add(rhs) as *const ();
    }
}

impl CapabilityPtr {
    pub fn as_ptr<T>(&self) -> *const T {
        self.ptr as *const T
    }

    /// Provenance note: may derive from a pointer other than the input to provide something with
    /// valid provenance to justify the other arguments.
    #[inline]
    pub fn new_with_metadata(
        ptr: *const (),
        _base: usize,
        _length: usize,
        _perms: MetaPermissions,
    ) -> Self {
        Self { ptr }
    }

    #[inline]
    pub fn map_or<U, F>(&self, default: U, f: F) -> U
    where
        F: FnOnce(&Self) -> U,
    {
        if self.ptr.is_null() {
            default
        } else {
            f(self)
        }
    }

    #[inline]
    pub fn map_or_else<U, D, F>(&self, default: D, f: F) -> U
    where
        D: FnOnce() -> U,
        F: FnOnce(&Self) -> U,
    {
        if self.ptr.is_null() {
            default()
        } else {
            f(self)
        }
    }
}
