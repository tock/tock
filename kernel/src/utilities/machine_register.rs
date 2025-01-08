// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Machine-specific register-sized type.
//!
//! This type holds exactly one machine register of data. This type should be
//! used when storing a value that is exactly the contents of a register with no
//! additional type information.
//!
//! Tock defines this as a custom type as there is currently (Nov 2024) no
//! suitable standard type for this purpose that is correct across all hardware
//! architectures Tock supports. The closest suitable type is `usize`. However,
//! `usize` only captures the size of data, not necessarily the full size of a
//! register. On many platforms these are the same, but on platforms with
//! ISA-level memory protection (e.g., CHERI), a register is larger than
//! `usize`.

use core::fmt::{Formatter, LowerHex, UpperHex};

/// [`MachineRegister`] is a datatype that can hold exactly the contents of a
/// register with no additional semantic information.
///
/// [`MachineRegister`] is useful for identifying when data within the Tock
/// kernel has no semantic meaning other than being the size of a register. In
/// the future it may be possible, useful, or necessary to change the
/// implementation of [`MachineRegister`], however, the semantics will remain.
/// No use of [`MachineRegister`] should assume a particular Rust implementation
/// or any semantics other this description.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct MachineRegister {
    /// We store the actual data as a rust pointer as a convenient way to hold
    /// an architecture-specific register worth of data.
    ///
    /// While represented as a pointer type, `value` has no semantic meaning as
    /// a pointer. Any uses of a [`MachineRegister`] as a pointer must be based
    /// other semantic understanding within Tock, and not from this type.
    value: *const (),
}

impl Default for MachineRegister {
    fn default() -> Self {
        Self {
            value: core::ptr::null(),
        }
    }
}

impl From<MachineRegister> for usize {
    /// Returns the contents of the register.
    #[inline]
    fn from(from: MachineRegister) -> Self {
        from.value as usize
    }
}

impl From<usize> for MachineRegister {
    /// Create a [`MachineRegister`] representation of a `usize`.
    #[inline]
    fn from(from: usize) -> Self {
        Self {
            value: from as *const (),
        }
    }
}

impl UpperHex for MachineRegister {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        UpperHex::fmt(&(self.value as usize), f)
    }
}

impl LowerHex for MachineRegister {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        LowerHex::fmt(&(self.value as usize), f)
    }
}
