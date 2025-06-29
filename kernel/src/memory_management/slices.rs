// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2025.

//! Slices of memories from the perspective of the memory management system.

use super::pointers::{Pointer, ValidVirtualPointer};

use crate::utilities::alignment::Alignment;
use crate::utilities::ordering::SmallerPair;
use crate::utilities::slices::NonEmptySlice;

use core::num::NonZero;

/// A non-empty slice of memory from the perspective of the memory management system.
pub struct Slice<'a, const IS_VIRTUAL: bool, const IS_MUTABLE: bool, T: Alignment>(
    NonEmptySlice<'a, IS_MUTABLE, T>,
);

/// A non-empty slice of physical memory from the perspective of the memory management system.
pub type PhysicalSlice<'a, const IS_MUTABLE: bool, T> = Slice<'a, false, IS_MUTABLE, T>;
/// A non-empty slice of immutable physical memory from the perspective of the memory management
/// system.
pub type ImmutablePhysicalSlice<'a, T> = PhysicalSlice<'a, false, T>;
/// A non-empty slice of mutable physical memory from the perspective of the memory management
/// system.
pub type MutablePhysicalSlice<'a, T> = PhysicalSlice<'a, true, T>;

/// A non-empty slice of virtual memory from the perspective of the memory management system.
pub type VirtualSlice<'a, const IS_MUTABLE: bool, T> = Slice<'a, true, IS_MUTABLE, T>;
/// A non-empty slice of immutable virtual memory from the perspective of the memory management
/// system.
pub type ImmutableVirtualSlice<'a, T> = VirtualSlice<'a, false, T>;
/// A non-empty slice of mutable virtual memory from the perspective of the memory management
/// system.
pub type MutableVirtualSlice<'a, T> = VirtualSlice<'a, true, T>;

/// A valid non-empty slice of virtual memory, that is, it belongs to either the kernel or a
/// process.
pub struct ValidVirtualSlice<'a, const IS_USER: bool, const IS_MUTABLE: bool, T>(
    VirtualSlice<'a, IS_MUTABLE, T>,
);

/// A non-empty slice of user virtual memory.
pub type UserVirtualSlice<'a, const IS_MUTABLE: bool, T> =
    ValidVirtualSlice<'a, true, IS_MUTABLE, T>;
/// A non-empty slice of user immutable virtual memory.
pub type ImmutableUserVirtualSlice<'a, T> = UserVirtualSlice<'a, false, T>;
/// A non-empty slice of user mutable virtual memory.
pub type MutableUserVirtualSlice<'a, T> = UserVirtualSlice<'a, true, T>;

/// A non-empty slice of kernel virtual memory.
pub type KernelVirtualSlice<'a, const IS_MUTABLE: bool, T> =
    ValidVirtualSlice<'a, false, IS_MUTABLE, T>;
/// A non-empty slice of kernel immutable virtual memory.
pub type ImmutableKernelVirtualSlice<'a, T> = KernelVirtualSlice<'a, false, T>;
/// A non-empty slice of kernel mutable virtual memory.
pub type MutableKernelVirtualSlice<'a, T> = KernelVirtualSlice<'a, true, T>;

impl<'a, const IS_VIRTUAL: bool, const IS_MUTABLE: bool, T: Alignment>
    Slice<'a, IS_VIRTUAL, IS_MUTABLE, T>
{
    /// # Safety
    ///
    /// The caller must ensure that:
    ///
    /// 1. No other reference to the memory covered by this slice exists.
    /// 2. The slice does not wrap around at the end of the address space.
    /// 3. The memory covered by the slice is valid for the <'a> lifetime.
    pub(crate) const unsafe fn from_raw_parts(
        pointer: Pointer<IS_VIRTUAL, IS_MUTABLE, T>,
        length: NonZero<usize>,
    ) -> Self {
        let non_empty_slice = NonEmptySlice::from_raw_parts(pointer.downgrade(), length);
        // SAFETY: `pointer` guarantees that the memory is of right type
        unsafe { Self::new(non_empty_slice) }
    }

    /// # Safety
    ///
    /// The caller must ensure that:
    ///
    /// 1. No other reference to the memory covered by this slice exists.
    /// 2. The memory covered by the slice is valid for the <'a> lifetime.
    pub unsafe fn new_start_end(pointers: SmallerPair<Pointer<IS_VIRTUAL, IS_MUTABLE, T>>) -> Self {
        let non_empty_slice = NonEmptySlice::new_start_end(pointers.downgrade());
        // SAFETY: `non_empty_slice` comes from `pointers`
        unsafe { Self::new(non_empty_slice) }
    }

    /// # Safety
    ///
    /// The caller must ensure that `non_empty_slice` is:
    ///
    /// 1. Physical if IS_VIRTUAL == false
    /// 2. Virtual if IS_VIRTUAL == true
    pub const unsafe fn new(non_empty_slice: NonEmptySlice<'a, IS_MUTABLE, T>) -> Self {
        Self(non_empty_slice)
    }

    const fn as_non_empty_slice(&self) -> &NonEmptySlice<'a, IS_MUTABLE, T> {
        &self.0
    }

    const fn into_non_empty_slice(self) -> NonEmptySlice<'a, IS_MUTABLE, T> {
        self.0
    }

    pub(crate) const fn get_starting_pointer(&self) -> &Pointer<IS_VIRTUAL, IS_MUTABLE, T> {
        let aligned_non_null_pointer = self.as_non_empty_slice().get_starting_pointer();
        // SAFETY: `aligned_non_null_pointer` comes from `self`
        unsafe { Pointer::new_ref(aligned_non_null_pointer) }
    }

    pub(crate) fn get_ending_pointer(&self) -> Pointer<IS_VIRTUAL, IS_MUTABLE, T> {
        let starting_pointer = self.get_starting_pointer();
        let length = self.get_length();
        // SAFETY: a slice may never wrap
        unsafe { starting_pointer.unchecked_add(length) }
    }

    pub(crate) const fn get_length(&self) -> NonZero<usize> {
        self.as_non_empty_slice().get_length()
    }

    fn into_starting_pointer(self) -> Pointer<IS_VIRTUAL, IS_MUTABLE, T> {
        let aligned_non_null_pointer = self.into_non_empty_slice().into_starting_pointer();
        // SAFETY: `aligned_non_null_pointer` comes from `self`
        unsafe { Pointer::new(aligned_non_null_pointer) }
    }

    pub(super) fn into_pointers(self) -> SmallerPair<Pointer<IS_VIRTUAL, IS_MUTABLE, T>> {
        let length = self.get_length();
        let starting_pointer = self.into_starting_pointer();
        // SAFETY: a slice cannot wrap
        let ending_pointer = unsafe { starting_pointer.unchecked_add(length) };
        // SAFETY: since a slice cannot wrap, `ending_pointer` > `starting_pointer`
        unsafe { SmallerPair::new_unchecked(starting_pointer, ending_pointer) }
    }

    pub(crate) fn split_at_checked(
        self,
        mid: NonZero<usize>,
    ) -> Result<(Self, Option<Self>), Self> {
        let non_empty_slice = self.into_non_empty_slice();
        let result = non_empty_slice.split_at_checked(mid);

        match result {
            Err(non_empty_slice) => {
                // SAFETY: `non_empty_slice` comes from `self`, so it has the right type
                let slice = unsafe { Self::new(non_empty_slice) };
                Err(slice)
            }
            Ok((left_non_empty_slice, optional_right_non_empty_slice)) => {
                // SAFETY: `left_non_empty_slice` comes from `self`, so it has the right type
                let left_slice = unsafe { Self::new(left_non_empty_slice) };
                let optional_right_slice = match optional_right_non_empty_slice {
                    None => None,
                    // SAFETY: `right_non_empty_slice` comes from `self`, so it has the right type
                    Some(right_non_empty_slice) => {
                        Some(unsafe { Self::new(right_non_empty_slice) })
                    }
                };

                Ok((left_slice, optional_right_slice))
            }
        }
    }
}

impl<'a, const IS_USER: bool, const IS_MUTABLE: bool, T: Alignment>
    ValidVirtualSlice<'a, IS_USER, IS_MUTABLE, T>
{
    /// # Safety
    ///
    /// The caller must ensure that:
    ///
    /// 1. No other reference to the memory covered by this slice exists.
    /// 2. The slice does not wrap around at the end of the address space.
    /// 3. The memory covered by the slice is valid for the <'a> lifetime.
    /// 4. The slice is virtual.
    pub(crate) const unsafe fn from_raw_parts(
        pointer: ValidVirtualPointer<IS_USER, IS_MUTABLE, T>,
        length: NonZero<usize>,
    ) -> Self {
        // SAFETY: the safety requirements are ensured by the caller.
        let virtual_slice = unsafe { VirtualSlice::from_raw_parts(pointer.downgrade(), length) };

        // SAFETY: `pointer` ensures that `virtual_slice` is of the right type.
        unsafe { Self::new(virtual_slice) }
    }

    /// # Safety
    ///
    /// The caller must ensure that `virtual_slice` is of the right type.
    const unsafe fn new(virtual_slice: VirtualSlice<'a, IS_MUTABLE, T>) -> Self {
        Self(virtual_slice)
    }

    const fn as_virtual_slice(&self) -> &VirtualSlice<'a, IS_MUTABLE, T> {
        &self.0
    }

    const fn into_virtual_slice(self) -> VirtualSlice<'a, IS_MUTABLE, T> {
        self.0
    }

    pub(crate) fn get_starting_pointer(&self) -> &ValidVirtualPointer<IS_USER, IS_MUTABLE, T> {
        let virtual_pointer = self.as_virtual_slice().get_starting_pointer();
        // SAFETY: `ValidVirtualPointer` is marked #[repr(transparent)] so it has the same memory
        // layout as `VirtualPointer`
        unsafe { &*core::ptr::from_ref(virtual_pointer).cast() }
    }

    pub(crate) fn get_ending_pointer(&self) -> ValidVirtualPointer<IS_USER, IS_MUTABLE, T> {
        let virtual_pointer = self.as_virtual_slice().get_ending_pointer();
        // SAFETY: `virtual_pointer` comes from `self`
        unsafe { ValidVirtualPointer::new(virtual_pointer) }
    }

    pub(crate) const fn get_length(&self) -> NonZero<usize> {
        self.as_virtual_slice().get_length()
    }

    pub(crate) fn split_at_checked(
        self,
        mid: NonZero<usize>,
    ) -> Result<(Self, Option<Self>), Self> {
        let virtual_slice = self.into_virtual_slice();
        let result = virtual_slice.split_at_checked(mid);

        match result {
            Err(virtual_slice) => {
                // SAFETY: `virtual_slice` comes from `self`, so it has the right type
                let slice = unsafe { Self::new(virtual_slice) };
                Err(slice)
            }
            Ok((left_virtual_slice, optional_right_virtual_slice)) => {
                // SAFETY: `left_virtual_slice` comes from `self`, so it has the right type
                let left_slice = unsafe { Self::new(left_virtual_slice) };
                let optional_right_slice = match optional_right_virtual_slice {
                    None => None,
                    // SAFETY: `right_virtual_slice` comes from `self`, so it has the right type
                    Some(right_virtual_slice) => Some(unsafe { Self::new(right_virtual_slice) }),
                };

                Ok((left_slice, optional_right_slice))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::memory_management::pages::Page4KiB;
    use crate::memory_management::pointers::{PhysicalPointer, UserVirtualPointer, VirtualPointer};
    use crate::utilities::misc::create_non_zero_usize;
    use crate::utilities::pointers::{ImmutablePointer, MutablePointer};

    #[test]
    fn test_physical_slice_split_at_checked() {
        let pointer = ImmutablePointer::new(0x80000000 as *const Page4KiB).unwrap();
        // SAFETY: let's assume 0x80000000 is a valid physical pointer
        let pointer = unsafe { PhysicalPointer::new(pointer) };
        let length = create_non_zero_usize(32);
        // SAFETY: let's assume the slice is valid
        let slice = unsafe { ImmutablePhysicalSlice::from_raw_parts(pointer, length) };

        let (slice, optional_leftover) = match slice.split_at_checked(create_non_zero_usize(2)) {
            Ok(result) => result,
            Err(_) => panic!("split_at_checked() should have succeeded"),
        };

        assert_eq!(0x80000000, slice.get_starting_pointer().get_address().get());
        assert_eq!(create_non_zero_usize(2), slice.get_length());

        let leftover = match optional_leftover {
            None => panic!("leftover cannot be None"),
            Some(leftover) => leftover,
        };

        assert_eq!(
            0x80002000,
            leftover.get_starting_pointer().get_address().get()
        );
        assert_eq!(create_non_zero_usize(30), leftover.get_length());

        let leftover = match leftover.split_at_checked(create_non_zero_usize(31)) {
            Err(leftover) => leftover,
            Ok(_) => panic!("split_at_checked() should have failed"),
        };

        let (slice, optional_leftover) = match leftover.split_at_checked(create_non_zero_usize(30))
        {
            Err(_) => panic!("split_at_checked() should have succeeded"),
            Ok(result) => result,
        };

        assert_eq!(0x80002000, slice.get_starting_pointer().get_address().get());
        assert_eq!(create_non_zero_usize(30), slice.get_length());
        assert!(optional_leftover.is_none());
    }

    #[test]
    fn test_virtual_slice_split_at_checked() {
        let pointer = MutablePointer::new(0x40000000 as *mut Page4KiB).unwrap();
        // SAFETY: let's assume 0x40000000 is a valid virtual pointer
        let pointer = unsafe { VirtualPointer::new(pointer) };
        // SAFETY: let's assume 0x40000000 is a valid user virtual pointer
        let pointer = unsafe { UserVirtualPointer::new(pointer) };
        let length = create_non_zero_usize(123);
        // SAFETY: let's assume the slice is valid
        let slice = unsafe { MutableUserVirtualSlice::from_raw_parts(pointer, length) };

        let (slice, optional_leftover) = match slice.split_at_checked(create_non_zero_usize(98)) {
            Ok(result) => result,
            Err(_) => panic!("split_at_checked() should have succeeded"),
        };

        assert_eq!(0x40000000, slice.get_starting_pointer().get_address().get());
        assert_eq!(create_non_zero_usize(98), slice.get_length());

        let leftover = match optional_leftover {
            None => panic!("leftover cannot be None"),
            Some(leftover) => leftover,
        };

        assert_eq!(
            0x40062000,
            leftover.get_starting_pointer().get_address().get()
        );
        assert_eq!(create_non_zero_usize(25), leftover.get_length());

        let leftover = match leftover.split_at_checked(create_non_zero_usize(26)) {
            Err(leftover) => leftover,
            Ok(_) => panic!("split_at_checked() should have failed"),
        };

        let (slice, optional_leftover) = match leftover.split_at_checked(create_non_zero_usize(25))
        {
            Err(_) => panic!("split_at_checked() should have succeeded"),
            Ok(result) => result,
        };

        assert_eq!(0x40062000, slice.get_starting_pointer().get_address().get());
        assert_eq!(create_non_zero_usize(25), slice.get_length());
        assert!(optional_leftover.is_none());
    }
}
