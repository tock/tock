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

use crate::utilities::capability_ptr::CapabilityPtr;

/// [`MachineRegister`] is a datatype that can hold exactly the contents of a
/// register with no additional semantic information.
///
/// We can define this as a [`CapabilityPtr`] as [`CapabilityPtr`] has the same
/// size.
///
/// [`MachineRegister`] is useful for identifying when data within the Tock
/// kernel has no semantic meaning other than being the size of a register. In
/// the future it may be possible, useful, or necessary to change the
/// implementation of [`MachineRegister`], however, the semantics will remain.
/// To reduce implementation complexity we simply use a type alias, but this
/// should not be construed as to suggest that [`MachineRegister`] will always
/// be implemented as a [`CapabilityPtr`].
pub type MachineRegister = CapabilityPtr;
