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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::memory_management::pages::Page4KiB;
    use crate::memory_management::pointers::PhysicalPointer;
    use crate::utilities::misc::create_non_zero_usize;
    use crate::utilities::pointers::MutablePointer;

    #[test]
    fn test_allocate() {
        let pointer = MutablePointer::new(0x303000 as *mut Page4KiB).unwrap();
        // SAFETY: let's assume 0x303000 is a valid physical pointer
        let pointer = unsafe { PhysicalPointer::new(pointer) };
        let length = create_non_zero_usize(16);
        // SAFETY: let's assume the slice is valid
        let slice = unsafe { MutablePhysicalSlice::from_raw_parts(pointer, length) };
        let allocator = StaticAllocator::new(slice);

        let allocation = allocator.allocate(create_non_zero_usize(1)).unwrap();
        assert_eq!(create_non_zero_usize(1), allocation.get_length());
        assert_eq!(
            0x303000,
            allocation.get_starting_pointer().get_address().get()
        );

        let allocation = allocator.allocate(create_non_zero_usize(11)).unwrap();
        assert_eq!(create_non_zero_usize(11), allocation.get_length());
        assert_eq!(
            0x304000,
            allocation.get_starting_pointer().get_address().get()
        );

        let result = allocator.allocate(create_non_zero_usize(5));
        assert!(result.is_err());

        let allocation = allocator.allocate(create_non_zero_usize(4)).unwrap();
        assert_eq!(create_non_zero_usize(4), allocation.get_length());
        assert_eq!(
            0x30F000,
            allocation.get_starting_pointer().get_address().get()
        );

        let result = allocator.allocate(create_non_zero_usize(4));
        assert!(result.is_err());
    }
}
