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
//! architectures Tock supports. The closest suitable type is `*mut ()`.
//! However, in hybrid CHERI `*mut ()` is smaller than a full register.

use core::fmt::{Formatter, LowerHex, UpperHex};

use super::capability_ptr::CapabilityPtr;

/// [`MachineRegister`] is a datatype that can hold exactly the contents of a
/// register with no additional semantic information.
///
/// [`MachineRegister`] is useful when you know data is the size of a register,
/// but not its true data type. In the future it may be possible, useful, or
/// necessary to change the implementation of [`MachineRegister`], however, the
/// semantics will remain. No use of [`MachineRegister`] should assume a
/// particular Rust implementation or any semantics other this description.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
#[repr(transparent)]
pub struct MachineRegister {
    // We store the actual data as a CapabilityPtr as a convenient way to hold
    // an architecture-specific register's worth of data. `value` may or may not
    // really be a CapabilityPtr: it may instead contain an integer.
    value: CapabilityPtr,
}

impl From<CapabilityPtr> for MachineRegister {
    /// Creates a [`MachineRegister`] containing this [`CapabilityPtr`],
    /// including its provenance.
    fn from(from: CapabilityPtr) -> Self {
        Self { value: from }
    }
}

// Note: `From<usize> for MachineRegister` is implemented in the capability_ptr
// module.

impl UpperHex for MachineRegister {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        UpperHex::fmt(&self.value, f)
    }
}

impl LowerHex for MachineRegister {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        LowerHex::fmt(&self.value, f)
    }
}

impl MachineRegister {
    /// Returns this [`MachineRegister`] as a [`CapabilityPtr`].
    ///
    /// If this [`MachineRegister`] contains a pointer with provenance and/or
    /// authority, the returned [`CapabilityPtr`] will have the same provenance
    /// and/or authority.
    pub fn as_capability_ptr(self) -> CapabilityPtr {
        self.value
    }

    /// Returns this [`MachineRegister`] as a [`usize`].
    ///
    /// This is intended for use on [`MachineRegister`]s created from a
    /// [`usize`], in which case the original [`usize`] will be returned. If
    /// this [`MachineRegister`] was created from a pointer, this returns the
    /// pointer's address (without exposing provenance).
    pub fn as_usize(self) -> usize {
        self.value.addr()
    }
}
