// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2025.

//! Static allocator.

use super::allocator::Allocator;

use super::super::slices::MutablePhysicalSlice;

use crate::utilities::cells::OptionalCell;

use core::num::NonZero;

/// A static allocator, i.e. an allocator that allocates memory without freeing it ever.
pub struct StaticAllocator<'a, Granule>(OptionalCell<MutablePhysicalSlice<'a, Granule>>);

impl<'a, Granule> StaticAllocator<'a, Granule> {
    pub const fn new(memory: MutablePhysicalSlice<'a, Granule>) -> Self {
        Self(OptionalCell::new(memory))
    }

    fn take(&self) -> Result<MutablePhysicalSlice<'a, Granule>, ()> {
        match self.0.take() {
            None => Err(()),
            Some(memory) => Ok(memory),
        }
    }

    fn split_at_mut_checked(
        &self,
        memory: MutablePhysicalSlice<'a, Granule>,
        mid: NonZero<usize>,
    ) -> Result<
        (
            MutablePhysicalSlice<'a, Granule>,
            Option<MutablePhysicalSlice<'a, Granule>>,
        ),
        (),
    > {
        memory.split_at_checked(mid).map_err(|memory| {
            self.0.insert(Some(memory));
        })
    }
}

impl<'a, Granule> Allocator<'a, Granule> for StaticAllocator<'a, Granule> {
    fn allocate(&self, count: NonZero<usize>) -> Result<MutablePhysicalSlice<'a, Granule>, ()> {
        let memory = self.take()?;
        let (left_subslice, right_subslice) = self.split_at_mut_checked(memory, count)?;
        self.0.insert(right_subslice);
        Ok(left_subslice)
    }
}
