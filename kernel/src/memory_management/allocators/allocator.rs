// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2025.

use super::super::slices::MutablePhysicalSlice;

use core::num::NonZero;

pub trait Allocator<'a, Granule> {
    fn allocate(&self, count: NonZero<usize>) -> Result<MutablePhysicalSlice<'a, Granule>, ()>;
}
