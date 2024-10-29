// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Google LLC 2024.

//! Defines the CapabilityPtr type

use core::fmt::{Formatter, LowerHex, UpperHex};
use core::ops::AddAssign;

/// A pointer with target specific metadata concerning validity or access rights.
///
/// This should be used any time the kernel wishes to grant authority to the user, or any time
/// the user should be required to prove validity of a pointer.
///
/// Values that are just raw addresses but imply nothing about a Rust object at that location
/// should be `usize`.
/// Values that are references, but do not cross the boundary between the user and the
/// kernel (or do cross the boundary but are merely informative and do not imply any rights)
/// can be `*const T` (or `&T` if the kernel knows they are valid).
/// Values that are references, and do need to cross the boundary, should be this type.
///
/// For example, `allow` grants authority to the kernel to access a buffer, so passes [CapabilityPtr]s.
/// Conversely, when a process communicates its stack location to the kernel it need not be
/// passed as a [CapabilityPtr], as the kernel does not access it.
///
/// [CapabilityPtr] is also assumed to be wide enough that it could contain a raw pointer (`*const ()`) or
/// A `usize`, possibly podding with extra bits. It is therefore an appropriate choice for the type
/// of a register that may contain any one of these in the syscall ABI at a point where it is not
/// yet clear which of these it is yet.
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
    Any,
    Read,
    Write,
    ReadWrite,
    Execute,
}

impl From<CapabilityPtr> for usize {
    #[inline]
    fn from(from: CapabilityPtr) -> Self {
        from.ptr as usize
    }
}

impl From<usize> for CapabilityPtr {
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

    /// Convert to a raw pointer, checking that metadata allows a particular set of permissions over
    /// a given number of bytes.
    /// If the metadata does not allow for this, returns null.
    /// If no such metadata exists, this succeeds.
    #[inline]
    pub fn as_ptr_checked<T>(&self, _length: usize, _perms: MetaPermissions) -> *const T {
        self.ptr as *const T
    }

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
