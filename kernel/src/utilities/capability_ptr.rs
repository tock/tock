// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Google LLC 2024.

//! Defines the CapabilityPtr type

use core::fmt::{Formatter, LowerHex, UpperHex};
use core::ops::AddAssign;
use core::ptr::null;

use super::machine_register::MachineRegister;

/// A pointer to userspace memory with implied authority.
///
/// A [`CapabilityPtr`] points to memory a userspace process may be permitted to
/// read, write, or execute. It is sized exactly to a CPU register that can pass
/// values between userspace and the kernel [^note1]. Operations on the pointer
/// may affect permissions, e.g. offsetting the pointer beyond the bounds of the
/// memory object may invalidate it.
///
/// [`CapabilityPtr`] should be used to store or pass a value between the
/// kernel and userspace that may represent a valid userspace reference,
/// when one party intends the other to access it.
///
/// [^note1]: Depending on the architecture, the size of a
/// [`CapabilityPtr`] may be a word size or larger, e.g., if registers
/// can store metadata such as access permissions.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct CapabilityPtr {
    ptr: *const (),
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

impl Default for CapabilityPtr {
    /// Returns a null CapabilityPtr.
    fn default() -> Self {
        Self { ptr: null() }
    }
}

impl From<usize> for CapabilityPtr {
    /// Constructs a [`CapabilityPtr`] with a given address but no authority or
    /// provenance.
    #[inline]
    fn from(from: usize) -> Self {
        Self {
            // Ideally this would be core::ptr::without_provenance(from), but
            // the CHERI toolchain is too old for without_provenance. This is
            // equivalent.
            ptr: null::<()>().with_addr(from),
        }
    }
}

// In addition to its publicly-documented capabilities, CapabilityPtr's
// implementation can also store integers. MachineRegister uses that ability to
// simplify its implementation. No other user of CapabilityPtr should rely on
// that ability.

impl From<usize> for MachineRegister {
    fn from(from: usize) -> Self {
        Self::from(CapabilityPtr::from(from))
    }
}

impl UpperHex for CapabilityPtr {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        UpperHex::fmt(&self.ptr.addr(), f)
    }
}

impl LowerHex for CapabilityPtr {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        LowerHex::fmt(&self.ptr.addr(), f)
    }
}

impl AddAssign<usize> for CapabilityPtr {
    /// Increments the address of a [`CapabilityPtr`]. If the pointer is offset
    /// past its bounds, its authority may be invalidated.
    #[inline]
    fn add_assign(&mut self, rhs: usize) {
        self.ptr = self.ptr.wrapping_byte_add(rhs);
    }
}

impl CapabilityPtr {
    /// Returns the address of this pointer. Does not expose provenance.
    pub fn addr(self) -> usize {
        self.ptr.addr()
    }

    /// Returns the pointer component of a [`CapabilityPtr`] but without any of the authority.
    pub fn as_ptr<T>(&self) -> *const T {
        self.ptr.cast()
    }

    /// Construct a [`CapabilityPtr`] from a raw pointer, with authority
    /// ranging over [`base`, `base + length`) and permissions `perms`.
    ///
    /// [`CapabilityPtr`]s can be the sole memory isolation primitive in the
    /// system, thus great care must be taken with this method, as errors can
    /// break Tock's isolation model.
    ///
    /// Provenance note: may derive from a pointer other than the input to
    /// provide something with valid provenance to justify the other arguments.
    ///
    /// # Safety
    ///
    /// NOTE: Hardware capability support is **experimental** in Tock. The
    /// intent of this type is to capture semantics general to any hardware
    /// capability implementation, however, there is no upstream support for
    /// any such hardware today. As such, the exact safety conditions may
    /// change in the future.
    ///
    /// Callers are responsible for ensuring that the authority conveyed by this
    /// pointer does not violate Tock's isolation model, under an assumption
    /// that hardware capabilities are the sole memory isolation mechanism.
    ///
    /// Authority is conveyed solely by the `base`, `length`, and `perms`.
    /// The value in `ptr` may be freely changed by untrusted code. When
    /// executing on a platform with hardware capability enforcement, callers
    /// may assume that on any attempt to use `ptr`, hardware will verify that:
    ///   - `ptr` is within the bounds [`base`, `base + length`).
    ///   - The access type is allowed by `perms`.
    ///
    /// Callers are responsible for restricting access to this pointer, i.e.,
    /// ensuring that a pointer which allows access to process memory is only
    /// given to the *correct* process.
    #[inline]
    pub unsafe fn new_with_authority(
        ptr: *const (),
        _base: usize,
        _length: usize,
        _perms: CapabilityPtrPermissions,
    ) -> Self {
        Self { ptr }
    }

    /// If the [`CapabilityPtr`] is null returns `default`, otherwise applies `f` to `self`.
    #[inline]
    pub fn map_or<U, F>(&self, default: U, f: F) -> U
    where
        F: FnOnce(&Self) -> U,
    {
        if self.ptr.is_null() { default } else { f(self) }
    }

    /// If the [`CapabilityPtr`] is null returns `default`, otherwise applies `f` to `self`.
    /// default is only evaluated if `self` is not null.
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
