// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Google LLC 2024.

//! Defines the CapabilityPtr type

use core::fmt::{Formatter, LowerHex, UpperHex};
use core::ops::AddAssign;
use core::ptr::null;

use crate::cheri::{cheri_perms, cptr, CPtrOps};
use crate::config::{CfgMatch, CONFIG};
use crate::TIfCfg;

use super::machine_register::MachineRegister;

// The inner type for CapabilityPtr, which it abstracts with a consistent interface
type InnerType = TIfCfg!(is_cheri, cptr, *const ());

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
    ptr: InnerType,
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
        Self {
            ptr: InnerType::new(crate::cheri::null(), null()),
        }
    }
}

impl From<usize> for CapabilityPtr {
    /// Constructs a [`CapabilityPtr`] with a given address but no authority or
    /// provenance.
    #[inline]
    fn from(from: usize) -> Self {
        Self {
            ptr: if CONFIG.is_cheri {
                // On non-cheri this is a useless convervstion
                #[allow(clippy::useless_conversion)]
                InnerType::new_true(from.into())
            } else {
                // Ideally this would be core::ptr::without_provenance(from), but
                // the CHERI toolchain is too old for without_provenance. This is
                // equivalent.
                InnerType::new_false(null::<()>().with_addr(from))
            },
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
        match self.ptr.get_match() {
            CfgMatch::True(cheri_ptr) => UpperHex::fmt(cheri_ptr, f),
            CfgMatch::False(ptr) => UpperHex::fmt(&ptr.addr(), f),
        }
    }
}

impl LowerHex for CapabilityPtr {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self.ptr.get_match() {
            CfgMatch::True(cheri_ptr) => LowerHex::fmt(cheri_ptr, f),
            CfgMatch::False(ptr) => LowerHex::fmt(&ptr.addr(), f),
        }
    }
}

impl AddAssign<usize> for CapabilityPtr {
    /// Increments the address of a [`CapabilityPtr`]. If the pointer is offset
    /// past its bounds, its authority may be invalidated.
    #[inline]
    fn add_assign(&mut self, rhs: usize) {
        self.ptr.map_mut(
            |cheri_ptr| cheri_ptr.add_assign(rhs),
            |ptr| *ptr = ptr.wrapping_byte_add(rhs),
        )
    }
}

/// Helper to convert abstract permissions to the bitmap that CHERI uses
fn cheri_perms_for(perms: CapabilityPtrPermissions) -> usize {
    match perms {
        CapabilityPtrPermissions::None => 0,
        CapabilityPtrPermissions::Read => cheri_perms::DEFAULT_R,
        CapabilityPtrPermissions::Write => cheri_perms::STORE,
        CapabilityPtrPermissions::ReadWrite => cheri_perms::DEFAULT_RW,
        CapabilityPtrPermissions::Execute => cheri_perms::EXECUTE,
    }
}

impl CapabilityPtr {
    /// Returns the address of this pointer. Does not expose provenance.
    pub fn addr(self) -> usize {
        self.as_ptr::<()>().addr()
    }

    /// Checks whether any that metadata that exists would allow this operation.
    ///
    /// A [`CapabilityPtr`] constructed with new_with_authority would return true for this method
    /// if specifying the same length (or shorter) with the same permissions (or fewer).
    ///
    /// This is over-approximate as a lack of any such metadata would result in returning true.
    ///
    /// This likely does not meet the requirements rust would have to covert to a reference and
    /// further justification should be present elsewhere if doing so in the kernel.
    ///
    /// If this function returns false, then it should likely not be allowed.
    pub fn is_valid_for_operation(&self, length: usize, perms: CapabilityPtrPermissions) -> bool {
        match self.ptr.get_match() {
            CfgMatch::True(cheri_ptr) => {
                cheri_ptr.is_valid_for_operation(length, cheri_perms_for(perms))
            }
            CfgMatch::False(_) => true,
        }
    }

    /// Returns the pointer component of a [`CapabilityPtr`] but without any of the authority.
    pub fn as_ptr<T>(&self) -> *const T {
        match self.ptr.get_match() {
            CfgMatch::True(cheri_ptr) => cheri_ptr.as_ptr(),
            CfgMatch::False(ptr) => *ptr,
        }
        .cast()
    }

    /// Returns the pointer component of a [`CapabilityPtr`] but without any of the authority only
    /// if valid for an operation of length with perms.
    ///
    /// See is_valid_for_operation.
    pub fn as_ptr_checked<T>(
        &self,
        length: usize,
        perms: CapabilityPtrPermissions,
    ) -> Result<*const T, ()> {
        if self.is_valid_for_operation(length, perms) {
            Ok(self.as_ptr())
        } else {
            Err(())
        }
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
    #[inline]
    pub unsafe fn new_with_authority(
        ptr: *const (),
        base: usize,
        length: usize,
        perms: CapabilityPtrPermissions,
    ) -> Self {
        Self {
            ptr: if CONFIG.is_cheri {
                let mut result = cptr::default();
                if perms == CapabilityPtrPermissions::Execute {
                    result.set_addr_from_pcc_restricted(ptr as usize, base, length);
                } else {
                    result.set_addr_from_ddc_restricted(
                        ptr as usize,
                        base,
                        length,
                        cheri_perms_for(perms),
                    );
                }
                InnerType::new_true(result)
            } else {
                InnerType::new_false(ptr)
            },
        }
    }

    /// If the [`CapabilityPtr`] is null returns `default`, otherwise applies `f` to `self`.
    #[inline]
    pub fn map_or<U, F>(&self, default: U, f: F) -> U
    where
        F: FnOnce(&Self) -> U,
    {
        if self.as_ptr::<()>().is_null() {
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
        if self.as_ptr::<()>().is_null() {
            default()
        } else {
            f(self)
        }
    }
}
