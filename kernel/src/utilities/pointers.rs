// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2025.

//! Support for pointers.

use super::alignment::{Alignment, AlwaysAligned};
use super::misc::{create_non_zero_usize, divide_non_zero_usize, modulo_non_zero_usize};

use core::cmp::Ordering;
use core::fmt::{Debug, Formatter};
use core::num::NonZero;
use core::ops::Sub;
use core::ptr::NonNull;

/// Pointer creation error.
#[derive(Debug)]
pub enum Error {
    Null,
    NotAligned,
}

/// A non-null, aligned pointer.
#[repr(transparent)]
pub struct Pointer<const IS_MUTABLE: bool, T: Alignment>(NonNull<T>);

/// A mutable, non-null, aligned pointer.
pub type MutablePointer<T> = Pointer<true, T>;
/// An immutable, non-null, aligned pointer.
pub type ImmutablePointer<T> = Pointer<false, T>;

impl<const IS_MUTABLE: bool, T: Alignment> Pointer<IS_MUTABLE, T> {
    /// # Safety
    ///
    /// The caller must ensure `non_null_pointer` is suitably aligned
    const unsafe fn new_unchecked_alignment(non_null_pointer: NonNull<T>) -> Self {
        Self(non_null_pointer)
    }

    pub fn new_non_null(non_null_pointer: NonNull<T>) -> Result<Self, ()> {
        let address = non_null_pointer.addr();

        if modulo_non_zero_usize(address, T::ALIGNMENT) == 0 {
            // SAFETY: because of the if condition, the pointer is suitably aligned
            let pointer = unsafe { Self::new_unchecked_alignment(non_null_pointer) };
            Ok(pointer)
        } else {
            Err(())
        }
    }

    pub const fn to_immutable(self) -> ImmutablePointer<T> {
        let non_null_pointer = self.to_non_null();
        // SAFETY: `self` guarantees that `non_null_pointer` is aligned.
        unsafe { ImmutablePointer::new_unchecked_alignment(non_null_pointer) }
    }

    pub const fn as_immutable(&self) -> &ImmutablePointer<T> {
        // SAFETY: `Pointer` is marked #[repr(transparent)], so its memory layout is the same for
        // both immutable and mutable pointers
        unsafe { &*core::ptr::from_ref(self).cast() }
    }

    const fn as_non_null(&self) -> &NonNull<T> {
        &self.0
    }

    const fn to_non_null(self) -> NonNull<T> {
        self.0
    }

    pub fn get_address(&self) -> NonZero<usize> {
        self.as_non_null().addr()
    }

    /*
    const fn infaillible_cast<U: AlwaysAligned>(self) -> Pointer<IS_MUTABLE, U> {
        let non_null_pointer = self.to_non_null();
        // SAFETY:
        //
        // 1. U: AlwaysAligned
        // 2. `non_null_pointer` comes from `self`, so it is not null.
        unsafe { Pointer::<IS_MUTABLE, U>::new_unchecked_alignment(non_null_pointer.cast()) }
    }
    */

    /// # Safety
    ///
    /// The addition must not overflow.
    pub(crate) unsafe fn unchecked_add(&self, count: NonZero<usize>) -> Self {
        let non_null_pointer = self.as_non_null();
        let raw_pointer = non_null_pointer.as_ptr();
        // CAST: core::ptr::addr() is not const
        let address = raw_pointer.addr();
        // SAFETY: the caller ensures that the addition cannot overflow.
        let new_address = unsafe { address.unchecked_add(count.get() * core::mem::size_of::<T>()) };
        let new_raw_pointer = raw_pointer.with_addr(new_address);
        // SAFETY: adding a positive count to a non-null pointer without wrapping results in a
        // non-null pointer
        let new_non_null_pointer = unsafe { NonNull::new_unchecked(new_raw_pointer) };
        // SAFETY: Adding count * size_of::<T>() bytes to the address of T-aligned pointer results
        // in an address that is also multiple of size_of::<T>()
        unsafe { Self::new_unchecked_alignment(new_non_null_pointer) }
    }

    pub fn checked_add(&self, count: NonZero<usize>) -> Result<Self, ()> {
        let non_null_pointer = self.as_non_null();
        let raw_pointer = non_null_pointer.as_ptr();
        let address = raw_pointer.addr();
        let new_raw_pointer = match address.checked_add(count.get() * core::mem::size_of::<T>()) {
            None => return Err(()),
            Some(new_address) => raw_pointer.with_addr(new_address),
        };
        // SAFETY: adding a positive count to a non-null pointer without wrapping results in a
        // non-null pointer
        let new_non_null_pointer = unsafe { NonNull::new_unchecked(new_raw_pointer) };
        // SAFETY: Adding count * size_of::<T>() bytes to the address of T-aligned pointer results
        // in an address that is also multiple of size_of::<T>()
        let new_pointer = unsafe { Self::new_unchecked_alignment(new_non_null_pointer) };
        Ok(new_pointer)
    }

    pub fn checked_sub(&self, count: NonZero<usize>) -> Result<Self, ()> {
        let non_null_pointer = self.as_non_null();
        let raw_pointer = non_null_pointer.as_ptr();
        let address = raw_pointer.addr();
        let new_raw_pointer = match address.checked_sub(count.get() * core::mem::size_of::<T>()) {
            None => return Err(()),
            Some(new_address) => raw_pointer.with_addr(new_address),
        };
        // SAFETY: adding a positive count to a non-null pointer without wrapping results in a
        // non-null pointer
        let new_non_null_pointer = unsafe { NonNull::new_unchecked(new_raw_pointer) };
        // SAFETY: Adding count * size_of::<T>() bytes to the address of T-aligned pointer results
        // in an address that is also multiple of size_of::<T>()
        let new_pointer = unsafe { Self::new_unchecked_alignment(new_non_null_pointer) };
        Ok(new_pointer)
    }

    pub fn checked_offset(&self, count: NonZero<isize>) -> Result<Self, ()> {
        let non_null_pointer = self.as_non_null();
        let raw_pointer = non_null_pointer.as_ptr();
        // CAST: TODO
        let address = raw_pointer.addr() as isize;
        // CAST: TODO
        let new_raw_pointer =
            match address.checked_add(count.get() * core::mem::size_of::<T>() as isize) {
                None => return Err(()),
                // CAST: TODO
                Some(new_address) => raw_pointer.with_addr(new_address as usize),
            };
        // SAFETY: adding a positive count to a non-null pointer without wrapping results in a
        // non-null pointer
        let new_non_null_pointer = unsafe { NonNull::new_unchecked(new_raw_pointer) };
        // SAFETY: Adding count * size_of::<T>() bytes to the address of T-aligned pointer results
        // in an address that is also multiple of size_of::<T>()
        let new_pointer = unsafe { Self::new_unchecked_alignment(new_non_null_pointer) };
        Ok(new_pointer)
    }

    /// # Safety
    ///
    /// Same safety requirements as `core::ptr::sub_ptr()` + `self` > `origin`
    pub unsafe fn distance_from_origin(&self, origin: &Self) -> NonZero<usize> {
        let inner_address = self.get_address();
        let origin_address = origin.get_address();

        // SAFETY: the caller ensures that `self` > `origin`
        let difference = unsafe { inner_address.get().unchecked_sub(origin_address.get()) };
        // SAFETY: the caller ensures that `self` > `origin`
        let difference = unsafe { NonZero::new_unchecked(difference) };
        let distance =
            divide_non_zero_usize(difference, create_non_zero_usize(core::mem::size_of::<T>()));
        // SAFETY: the caller ensures that `self` > `origin`
        unsafe { NonZero::new_unchecked(distance) }
    }

    pub const unsafe fn offset_from(&self, origin: &Self) -> isize {
        let inner = self.as_non_null();
        let origin_inner = origin.as_non_null();

        // SAFETY: the distance between the two pointers is an exact multiple of the size of T
        unsafe { inner.offset_from(*origin_inner) }
    }

    pub fn cast<U: Alignment>(self) -> Result<Pointer<IS_MUTABLE, U>, Self> {
        let inner = self.to_non_null();
        let casted_inner = inner.cast();
        Pointer::new_non_null(casted_inner).map_err(|()| {
            let inner = casted_inner.cast();
            // SAFETY: casting preserves alignment
            unsafe { Self::new_unchecked_alignment(inner) }
        })
    }
}

impl<const IS_MUTABLE: bool, U: AlwaysAligned> Pointer<IS_MUTABLE, U> {
    pub fn new_from_non_null_byte(non_null_pointer: NonNull<U>) -> Self {
        // SAFETY: U is always aligned
        unsafe { Self::new_unchecked_alignment(non_null_pointer) }
    }
}

impl<T: Alignment> MutablePointer<T> {
    /// # Safety
    ///
    /// The caller must ensure that `pointer` is non-null and suitably aligned.
    pub(super) const unsafe fn new_unchecked(pointer: *mut T) -> Self {
        // SAFETY: the caller ensures that `pointer` is not null
        let non_null_pointer = unsafe { NonNull::new_unchecked(pointer) };
        // SAFETY: the caller ensures that `pointer is aligned
        unsafe { Self::new_unchecked_alignment(non_null_pointer) }
    }

    pub fn new(pointer: *mut T) -> Result<Self, Error> {
        let non_null_pointer = match NonNull::new(pointer) {
            None => return Err(Error::Null),
            Some(non_null_pointer) => non_null_pointer,
        };

        Self::new_non_null(non_null_pointer).map_err(|()| Error::NotAligned)
    }

    pub fn new_from_ref(reference: &mut T) -> Self {
        let pointer = core::ptr::from_mut(reference);
        // SAFETY: a reference is always suitably aligned and non-null
        unsafe { Self::new_unchecked(pointer) }
    }

    pub const fn to_raw(self) -> *mut T {
        self.to_non_null().as_ptr()
    }

    /// # Safety
    ///
    /// The two pointers must not be used at the same time.
    pub(crate) const unsafe fn as_raw(&self) -> *mut T {
        self.as_non_null().as_ptr()
    }

    /// # Safety
    ///
    /// `self` must be valid for writes and no reference to the memory pointed by `self` must exist
    pub(crate) fn write(&mut self, value: T) {
        // SAFETY:
        //
        // 1. the constructor ensures that the pointer is suitably aligned.
        // 2. the constructor ensures that the pointer is not null.
        // 3. the method's precondition ensures that `self` points to writeable memory.
        // 4. the method's precondition ensures that no reference to the memory pointed by `self`
        //    exists.
        unsafe { self.0.write(value) }
    }
}

impl<T: Alignment> ImmutablePointer<T> {
    /// # Safety
    ///
    /// The caller must ensure that `pointer` is non-null and suitably aligned.
    pub(super) const unsafe fn new_unchecked(pointer: *const T) -> Self {
        // SAFETY: the caller ensures that `pointer` is not null
        let non_null_pointer = unsafe { NonNull::new_unchecked(pointer.cast_mut()) };
        // SAFETY: the caller ensures that `pointer is aligned
        unsafe { Self::new_unchecked_alignment(non_null_pointer) }
    }

    pub fn new(pointer: *const T) -> Result<Self, Error> {
        let raw_mutable_pointer = pointer.cast_mut();

        MutablePointer::new(raw_mutable_pointer)
            .map(|mutable_pointer| mutable_pointer.to_immutable())
    }

    pub fn new_from_ref(reference: &T) -> Self {
        let pointer = core::ptr::from_ref(reference);
        // SAFETY: a reference is always suitably aligned and non-null
        unsafe { Self::new_unchecked(pointer) }
    }

    pub const fn to_raw(self) -> *const T {
        self.to_non_null().as_ptr().cast_const()
    }

    /// # Safety
    ///
    /// The two pointers must not be used at the same time.
    pub(crate) const unsafe fn as_raw(&self) -> *const T {
        self.as_non_null().as_ptr().cast_const()
    }
}

impl<const IS_MUTABLE: bool, T: Alignment> PartialEq for Pointer<IS_MUTABLE, T> {
    fn eq(&self, other: &Self) -> bool {
        self.as_non_null() == other.as_non_null()
    }
}

impl<const IS_MUTABLE: bool, T: Alignment> Eq for Pointer<IS_MUTABLE, T> {}

impl<const IS_MUTABLE: bool, T: Alignment> PartialOrd for Pointer<IS_MUTABLE, T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.as_non_null().cmp(other.as_non_null()))
    }
}

impl<const IS_MUTABLE: bool, T: Alignment> Ord for Pointer<IS_MUTABLE, T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_non_null().cmp(other.as_non_null())
    }
}

impl<const IS_MUTABLE: bool, T: Alignment> Sub for &Pointer<IS_MUTABLE, T> {
    type Output = isize;

    fn sub(self, other: Self) -> Self::Output {
        // SAFETY: `Pointer` guarantees that the distance between any pointer cannot exceed
        // isize::MAX bytes.
        unsafe { self.as_non_null().offset_from(*other.as_non_null()) }
    }
}

impl<const IS_MUTABLE: bool, T: Alignment> Debug for Pointer<IS_MUTABLE, T> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "{:?}", self.as_non_null())
    }
}

impl<const IS_MUTABLE: bool, T: Alignment> Clone for Pointer<IS_MUTABLE, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<const IS_MUTABLE: bool, T: Alignment> Copy for Pointer<IS_MUTABLE, T> {}

/// An aligned pointer.
pub enum NullablePointer<const IS_MUTABLE: bool, T: Alignment> {
    Null,
    NonNull(Pointer<IS_MUTABLE, T>),
}

/// An immutable aligned pointer.
pub type ImmutableNullablePointer<T> = NullablePointer<false, T>;
/// A mutable aligned pointer.
pub type MutableNullablePointer<T> = NullablePointer<true, T>;

impl<const IS_MUTABLE: bool, T: Alignment> NullablePointer<IS_MUTABLE, T> {
    pub const fn new_null() -> Self {
        NullablePointer::Null
    }

    /*
    const fn new_non_null(pointer: Pointer<IS_MUTABLE, T>) -> Self {
        NullablePointer::NonNull(pointer)
    }
    */
}
