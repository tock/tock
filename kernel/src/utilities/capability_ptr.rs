// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Google LLC 2024.

//! Defines the CapabilityPtr type

use core::fmt::{Formatter, LowerHex, UpperHex};
use core::ops::AddAssign;

use super::machine_register::MachineRegister;

/// A pointer to userspace memory with implied authority.
///
/// A [`CapabilityPtr`] points to memory a userspace process may be
/// permitted to read, write, or execute. It is sized exactly to a
/// CPU register that can pass values between userspace and the kernel.
/// Because it is register sized, [`CapabilityPtr`] is guaranteed to be
/// at least the size of a word ([usize]) [^note1]. Operations on the
/// pointer may affect permissions, e.g. offsetting the pointer beyond
/// the bounds of the memory object invalidates it. Like a `*const
/// ()`, a [`CapabilityPtr`] may also "hide" information by storing a
/// word of data with no memory access permissions.
///
/// [`CapabilityPtr`] should be used to store or pass a value between the
/// kernel and userspace that may represent a valid userspace reference,
/// when one party intends the other to access it.
///
/// [^note1]: Depending on the architecture, the size of a
/// [`CapabilityPtr`] may be a word size or larger, e.g., if registers
/// can store metadata such as access permissions.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
#[repr(transparent)]
pub struct CapabilityPtr {
    ptr: MachineRegister,
}

/// Permission sets a [`CapabilityPtr`] may grant.
/// These may not be enforced or exist on a given platform.
#[derive(Copy, Clone, PartialEq)]
pub enum CapabilityPtrPermissions {
    None,
    Read,
    Write,
    ReadWrite,
    Execute,
}

impl From<CapabilityPtr> for usize {
    /// Returns the address of the [`CapabilityPtr`].
    /// Provenance note: may not expose provenance.
    #[inline]
    fn from(from: CapabilityPtr) -> Self {
        from.ptr.into()
    }
}

impl From<usize> for CapabilityPtr {
    /// Constructs a [`CapabilityPtr`] with a given address and no authority
    ///
    /// Provenance note: may have null provenance.
    #[inline]
    fn from(from: usize) -> Self {
        Self { ptr: from.into() }
    }
}

impl From<MachineRegister> for CapabilityPtr {
    /// Convert a register to a [`CapabilityPtr`].
    #[inline]
    fn from(from: MachineRegister) -> Self {
        Self { ptr: from }
    }
}

impl UpperHex for CapabilityPtr {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        UpperHex::fmt(&self.ptr, f)
    }
}

impl LowerHex for CapabilityPtr {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        LowerHex::fmt(&self.ptr, f)
    }
}

impl AddAssign<usize> for CapabilityPtr {
    /// Increments the address of a [`CapabilityPtr`]
    #[inline]
    fn add_assign(&mut self, rhs: usize) {
        self.ptr =
            ((usize::from(self.ptr) as *const u8).wrapping_add(rhs) as *const () as usize).into();
    }
}

impl CapabilityPtr {
    /// Returns the pointer component of a [`CapabilityPtr`] but without any of the authority.
    pub fn as_ptr<T>(&self) -> *const T {
        usize::from(self.ptr) as *const T
    }

    /// Construct a [`CapabilityPtr`] from a raw pointer, with authority ranging over
    /// [`base`, `base + length`) and permissions `perms`.
    ///
    /// Provenance note: may derive from a pointer other than the input to provide something with
    /// valid provenance to justify the other arguments.
    ///
    /// ## Safety
    ///
    /// Constructing a [`CapabilityPtr`] with metadata may convey authority to
    /// dereference this pointer, such as in userspace. When these pointers
    /// serve as the only memory isolation primitive in the system, this method
    /// can thus break Tock's isolation model. As semi-trusted kernel code can
    /// name this type and method, it is thus marked as `unsafe`.
    ///
    // TODO: Once Tock supports hardware that uses the [`CapabilityPtr`]'s
    // metdata to convey authority, this comment should incorporate the exact
    // safety conditions of this function.
    #[inline]
    pub unsafe fn new_with_authority(
        ptr: *const (),
        _base: usize,
        _length: usize,
        _perms: CapabilityPtrPermissions,
    ) -> Self {
        Self {
            ptr: (ptr as usize).into(),
        }
    }

    /// If the [`CapabilityPtr`] is null returns `default`, otherwise applies `f` to `self`.
    #[inline]
    pub fn map_or<U, F>(&self, default: U, f: F) -> U
    where
        F: FnOnce(&Self) -> U,
    {
        if usize::from(self.ptr) == 0 {
            default
        } else {
            f(self)
        }
    }

    /// If the [`CapabilityPtr`] is null returns `default`, otherwise applies `f` to `self`.
    /// default is only evaluated if `self` is not null.
    #[inline]
    pub fn map_or_else<U, D, F>(&self, default: D, f: F) -> U
    where
        D: FnOnce() -> U,
        F: FnOnce(&Self) -> U,
    {
        if usize::from(self.ptr) == 0 {
            default()
        } else {
            f(self)
        }
    }
}
