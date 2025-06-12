// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2025.

//! Support for type alignment.

use super::misc::create_non_zero_usize;

use core::num::NonZero;

/// Type whose alignment is ALIGNMENT
pub trait Alignment {
    const ALIGNMENT: NonZero<usize>;
}

impl<T: Sized> Alignment for T {
    const ALIGNMENT: NonZero<usize> = create_non_zero_usize(core::mem::align_of::<Self>());
}

/// Type whose alignment is 1
pub trait AlwaysAligned: Alignment {}

impl AlwaysAligned for u8 {}
impl AlwaysAligned for i8 {}
