// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2025.

//! Memory allocators.

use super::super::slices::MutablePhysicalSlice;

use core::num::NonZero;

/// Memory allocator trait.
pub trait Allocator<'a, Granule> {
    /// Allocate `count` granules of memory.
    fn allocate(&self, count: NonZero<usize>) -> Result<MutablePhysicalSlice<'a, Granule>, ()>;
}
