// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2025.

use super::allocator::Allocator;

use super::super::slices::MutablePhysicalSlice;

use crate::utilities::cells::OptionalCell;

use core::num::NonZero;

pub struct StaticAllocator<'a, Granule>(OptionalCell<MutablePhysicalSlice<'a, Granule>>);

impl<'a, Granule> StaticAllocator<'a, Granule> {
    pub const fn new(memory: MutablePhysicalSlice<'a, Granule>) -> Self {
        Self(OptionalCell::new(memory))
    }

    fn take(&self) -> Result<MutablePhysicalSlice<'a, Granule>, ()> {
        match self.0.take() {
            None => return Err(()),
            Some(memory) => Ok(memory),
        }
    }

    fn split_at_mut_checked(
        &self,
        memory: MutablePhysicalSlice<'a, Granule>,
        mid: NonZero<usize>,
    ) -> Result<(MutablePhysicalSlice<'a, Granule>, Option<MutablePhysicalSlice<'a, Granule>>), ()> {
        memory.split_at_checked(mid).map_err(|memory| {
            self.0.insert(Some(memory));
            ()
        })
    }
}

impl<'a, Granule> Allocator<'a, Granule> for StaticAllocator<'a, Granule> {
    fn allocate(
        &self,
        count: NonZero<usize>,
    ) -> Result<MutablePhysicalSlice<'a, Granule>, ()> {
        let memory = self.take()?;
        let (left_subslice, right_subslice) = self.split_at_mut_checked(memory, count)?;
        self.0.insert(right_subslice);
        Ok(left_subslice)
    }

    fn allocate_from(
        &self,
        _starting_address: usize,
        _count: NonZero<usize>,
    ) -> Result<MutablePhysicalSlice<'a, Granule>, ()> {
        todo!()
        /*
        let memory = self.take()?;
        let memory_starting_address = memory.get_starting_address();

        let offset = match starting_address.checked_sub(memory_starting_address) {
            None => {
                self.0.insert(Some(memory));
                return Err(());
            }
            Some(offset) => offset,
        };

        let (_unusued_slice, used_slice) = self.split_at_mut_checked(memory, offset)?;
        let (allocated_slice, remaining_slice) = self.split_at_mut_checked(used_slice, count.get())?;
        self.0.insert(Some(remaining_slice));

        Ok(allocated_slice)
        */
    }
}
