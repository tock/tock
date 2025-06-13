// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2025.

//! Support for slices.

use super::alignment::Alignment;
use super::ordering::SmallerPair;
use super::pointers::{ImmutablePointer, MutablePointer, Pointer};

use core::marker::PhantomData;
use core::num::NonZero;

/// A non-empty slice.
pub struct NonEmptySlice<'a, const IS_MUTABLE: bool, T: Alignment> {
    pointer: Pointer<IS_MUTABLE, T>,
    length: NonZero<usize>,
    phantom_data: PhantomData<&'a ()>,
}

/// An immutable non-empty slice.
pub type NonEmptyImmutableSlice<'a, T> = NonEmptySlice<'a, false, T>;
/// A mutable empty slice.
pub type NonEmptyMutableSlice<'a, T> = NonEmptySlice<'a, true, T>;

impl<const IS_MUTABLE: bool, T: Alignment> NonEmptySlice<'_, IS_MUTABLE, T> {
    /// # Safety
    ///
    /// 1. the slice must not wrap around
    /// 2. the memory covered by the slice must be valid for the 'a lifetime
    /// 3. no other reference to the memory covered by the slice must exist
    pub(crate) const unsafe fn from_raw_parts(
        pointer: Pointer<IS_MUTABLE, T>,
        length: NonZero<usize>,
    ) -> Self {
        Self {
            pointer,
            length,
            phantom_data: PhantomData,
        }
    }

    /// # Safety
    ///
    /// The caller must ensure that:
    ///
    /// 1. No other reference to the memory covered by this slice exists.
    /// 2. The memory covered by the slice is valid for the <'a> lifetime.
    pub unsafe fn new_start_end(pointers: SmallerPair<Pointer<IS_MUTABLE, T>>) -> Self {
        // SAFETY: because of the previous if, `end` > `start`
        let length = pointers.compute_difference();
        let start = pointers.to_smaller();

        // SAFETY: the caller ensures that no other reference exists while the slice is available
        unsafe { Self::from_raw_parts(start, length) }
    }

    pub(crate) const fn into_starting_pointer(self) -> Pointer<IS_MUTABLE, T> {
        self.pointer
    }

    pub const fn get_starting_pointer(&self) -> &Pointer<IS_MUTABLE, T> {
        &self.pointer
    }

    pub const fn get_length(&self) -> NonZero<usize> {
        self.length
    }

    pub(crate) fn split_at_checked(
        self,
        mid: NonZero<usize>,
    ) -> Result<(Self, Option<Self>), Self> {
        let length = self.get_length();

        let difference = match length.get().checked_sub(mid.get()) {
            None => return Err(self),
            Some(difference) => difference,
        };

        let right_length = match NonZero::new(difference) {
            None => return Ok((self, None)),
            Some(right_length) => right_length,
        };

        let left_pointer = self.into_starting_pointer();
        // SAFETY: a slice cannot wrap around and since mid < length,
        // left_pointer.wrapping_add(mid) does not overlap.
        let right_pointer = unsafe { left_pointer.unchecked_add(mid) };

        // SAFETY: a slice cannot wrap around and since mid < length,
        // left_pointer.wrapping_add(mid) does not overlap.
        let left_slice = unsafe { Self::from_raw_parts(left_pointer, mid) };
        // SAFETY: a slice cannot wrap around and since
        // right_pointer.wrapping_add(right_length) == left_pointer.wrapping_add(length), the
        // result slice does not wrap around
        let right_slice = unsafe { Self::from_raw_parts(right_pointer, right_length) };

        Ok((left_slice, Some(right_slice)))
    }

    /*
    pub(crate) fn consume(self) -> (Pointer<IS_MUTABLE, T>, NonZero<usize>) {
        (self.pointer, self.length)
    }
    */
}

impl<'a, T: Alignment> NonEmptyMutableSlice<'a, T> {
    pub fn new(slice: &'a mut [T]) -> Result<Self, ()> {
        let length = slice.len();
        let non_zero_length = match NonZero::new(length) {
            None => return Err(()),
            Some(non_zero_length) => non_zero_length,
        };
        let raw_pointer = slice.as_mut_ptr();
        // SAFETY: a reference is always suitably aligned and non null.
        let pointer = unsafe { MutablePointer::new_unchecked(raw_pointer) };
        // SAFETY: both parts come from `slice` which guarantees they are valid.
        let non_empty_slice = unsafe { Self::from_raw_parts(pointer, non_zero_length) };

        Ok(non_empty_slice)
    }
}

impl<'a, T: Alignment> NonEmptyImmutableSlice<'a, T> {
    pub fn new(slice: &'a [T]) -> Result<Self, ()> {
        let length = slice.len();
        let non_zero_length = match NonZero::new(length) {
            None => return Err(()),
            Some(non_zero_length) => non_zero_length,
        };
        let raw_pointer = slice.as_ptr();
        // SAFETY: a reference is always suitably aligned and non null.
        let pointer = unsafe { ImmutablePointer::new_unchecked(raw_pointer) };
        // SAFETY: both parts come from `slice` which guarantees they are valid.
        let non_empty_slice = unsafe { Self::from_raw_parts(pointer, non_zero_length) };

        Ok(non_empty_slice)
    }
}
