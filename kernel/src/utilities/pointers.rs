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
#[derive(Debug, PartialEq, Eq)]
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
        let new_non_null_pointer = match NonNull::new(new_raw_pointer) {
            None => return Err(()),
            Some(new_non_null_pointer) => new_non_null_pointer,
        };
        // SAFETY: Adding count * size_of::<T>() bytes to the address of T-aligned pointer results
        // in an address that is also multiple of size_of::<T>()
        let new_pointer = unsafe { Self::new_unchecked_alignment(new_non_null_pointer) };
        Ok(new_pointer)
    }

    pub fn checked_offset(&self, count: NonZero<isize>) -> Result<Self, ()> {
        let raw_value = count.get();

        if raw_value < 0 {
            match raw_value.checked_neg() {
                None => Err(()),
                Some(negated_raw_value) => {
                    // CAST: because of the if condition,
                    // raw_value < 0 => negated_raw_value = -raw_value > 0
                    let positive_raw_value = negated_raw_value as usize;
                    // SAFETY: count != 0 => -count != 0
                    let new_count = unsafe { NonZero::new_unchecked(positive_raw_value) };
                    self.checked_sub(new_count)
                }
            }
        } else {
            // CAST: because of the if condition, count > 0
            let positive_raw_value = raw_value as usize;
            // SAFETY: count != 0
            let new_count = unsafe { NonZero::new_unchecked(positive_raw_value) };
            self.checked_add(new_count)
        }
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

    /// # Safety
    ///
    /// Same requirements as `core::ptr::offset_from()`
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
    pub(crate) unsafe fn write(&mut self, value: T) {
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

    /// # Safety
    ///
    /// `self` must be valid for reads and no reference to the memory pointed by `self` must exist
    pub(crate) unsafe fn read(&self) -> T {
        // SAFETY:
        //
        // 1. the constructor ensures that the pointer is suitably aligned.
        // 2. the constructor ensures that the pointer is not null.
        // 3. the method's precondition ensures that `self` points to writeable memory.
        // 4. the method's precondition ensures that no reference to the memory pointed by `self`
        //    exists.
        unsafe { self.0.read() }
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::utilities::misc::create_non_zero_isize;

    #[test]
    fn test_immutable_null() {
        let result = ImmutablePointer::new(0 as *const u32);
        assert_eq!(Err(Error::Null), result);
    }

    #[test]
    fn test_mutable_null() {
        let result = MutablePointer::new(0 as *mut u32);
        assert_eq!(Err(Error::Null), result);
    }

    #[test]
    fn test_immutable_unaligned() {
        let result = ImmutablePointer::new(1 as *const u32);
        assert_eq!(Err(Error::NotAligned), result);

        let result = ImmutablePointer::new(2 as *const u32);
        assert_eq!(Err(Error::NotAligned), result);

        let result = ImmutablePointer::new(3 as *const u32);
        assert_eq!(Err(Error::NotAligned), result);
    }

    #[test]
    fn test_mutable_unaligned() {
        let result = MutablePointer::new(1 as *mut u32);
        assert_eq!(Err(Error::NotAligned), result);

        let result = MutablePointer::new(2 as *mut u32);
        assert_eq!(Err(Error::NotAligned), result);

        let result = MutablePointer::new(3 as *mut u32);
        assert_eq!(Err(Error::NotAligned), result);
    }

    #[test]
    fn test_immutable_ok() {
        let pointer = ImmutablePointer::new(0x200000 as *const u32).unwrap();
        assert_eq!(create_non_zero_usize(0x200000), pointer.get_address());
    }

    #[test]
    fn test_mutable_ok() {
        let pointer = MutablePointer::new(0x200000 as *mut u32).unwrap();
        assert_eq!(create_non_zero_usize(0x200000), pointer.get_address());
    }

    #[test]
    fn test_mutable_cast_to_immutable() {
        let pointer = MutablePointer::new(0x200008 as *mut u64).unwrap();
        assert_eq!(0x200008, pointer.as_immutable().get_address().get());
    }

    #[test]
    fn test_cast_to_lower_alignment() {
        let pointer = MutablePointer::new(0x200008 as *mut u64).unwrap();
        let pointer = pointer.cast::<u32>().unwrap();
        assert_eq!(0x200008, pointer.get_address().get());
    }

    #[test]
    fn test_cast_lower_alignment() {
        let pointer = MutablePointer::new(0x200008 as *mut u64).unwrap();
        let pointer = pointer.cast::<u32>().unwrap();
        assert_eq!(0x200008, pointer.get_address().get());
    }

    #[test]
    fn test_bad_cast_greater_alignment() {
        let pointer = MutablePointer::new(0x200004 as *mut u32).unwrap();
        assert!(pointer.cast::<u64>().is_err());
    }

    #[test]
    fn test_good_cast_greater_alignment() {
        let pointer = MutablePointer::new(0x200008 as *mut u32).unwrap();
        let pointer = pointer.cast::<u64>().unwrap();
        assert_eq!(0x200008, pointer.get_address().get());
    }

    #[test]
    fn test_checked_add_ok() {
        let pointer = ImmutablePointer::new(0x200004 as *const u32).unwrap();
        let pointer = pointer.checked_add(create_non_zero_usize(3)).unwrap();
        assert_eq!(0x200010, pointer.get_address().get());
    }

    #[test]
    fn test_checked_add_overflow() {
        let pointer = ImmutablePointer::new((usize::MAX - 79) as *const u64).unwrap();
        let pointer = pointer.checked_add(create_non_zero_usize(9)).unwrap();
        assert_eq!(usize::MAX - 7, pointer.get_address().get());

        let result = pointer.checked_add(create_non_zero_usize(1));
        assert!(result.is_err());

        let result = pointer.checked_add(create_non_zero_usize(12));
        assert!(result.is_err());
    }

    #[test]
    fn test_unchecked_add_ok() {
        let pointer = ImmutablePointer::new(0x200004 as *const u32).unwrap();
        // SAFETY:
        //
        // 1. The resulting address does not overflow.
        // 2. Let's assume the two pointers belong to the same object.
        let pointer = unsafe { pointer.unchecked_add(create_non_zero_usize(0x1000)) };
        assert_eq!(0x204004, pointer.get_address().get());
    }

    #[test]
    fn test_checked_sub_ok() {
        let pointer = MutablePointer::new(0x20 as *mut u64).unwrap();
        let pointer = pointer.checked_sub(create_non_zero_usize(2)).unwrap();
        assert_eq!(0x10, pointer.get_address().get());
    }

    #[test]
    fn test_checked_sub_underflow() {
        let pointer = MutablePointer::new(0x20 as *mut u64).unwrap();
        let pointer = pointer.checked_sub(create_non_zero_usize(3)).unwrap();
        assert_eq!(0x08, pointer.get_address().get());

        let result = pointer.checked_sub(create_non_zero_usize(1));
        assert!(result.is_err());

        let result = pointer.checked_sub(create_non_zero_usize(12));
        assert!(result.is_err());
    }

    #[test]
    fn test_checked_offset_ok() {
        let pointer = ImmutablePointer::new(0x200003 as *const u8).unwrap();
        let pointer = pointer.checked_offset(create_non_zero_isize(2)).unwrap();
        assert_eq!(0x200005, pointer.get_address().get());

        let pointer = pointer.checked_offset(create_non_zero_isize(-5)).unwrap();
        assert_eq!(0x200000, pointer.get_address().get());
    }

    #[test]
    fn test_checked_offset_overflow() {
        let pointer = ImmutablePointer::new((usize::MAX - 1) as *const u8).unwrap();
        let pointer = pointer.checked_offset(create_non_zero_isize(1)).unwrap();
        assert_eq!(usize::MAX, pointer.get_address().get());

        let result = pointer.checked_offset(create_non_zero_isize(1));
        assert!(result.is_err());

        let result = pointer.checked_offset(create_non_zero_isize(0x1000));
        assert!(result.is_err());
    }

    #[test]
    fn test_checked_offset_underflow() {
        let pointer = ImmutablePointer::new(100 as *const u8).unwrap();
        let result = pointer.checked_offset(create_non_zero_isize(isize::MIN));
        assert!(result.is_err());

        let pointer = pointer.checked_offset(create_non_zero_isize(-99)).unwrap();
        assert_eq!(1, pointer.get_address().get());

        let result = pointer.checked_offset(create_non_zero_isize(-1));
        assert!(result.is_err());

        let result = pointer.checked_offset(create_non_zero_isize(-400));
        assert!(result.is_err());
    }

    #[test]
    fn test_new_from_immutable_ref() {
        let value = 1234u32;
        let pointer = ImmutablePointer::new_from_ref(&value);
        // SAFETY: pointer is readable and `value` is no longer in use.
        let value = unsafe { pointer.read() };
        assert_eq!(1234, value);

        let pointer = pointer.clone();
        // SAFETY: pointer is readable and `value` is no longer in use.
        let value = unsafe { pointer.read() };
        assert_eq!(1234, value);
    }

    #[test]
    fn test_new_from_mutable_ref() {
        let mut value = 1234u32;
        let mut pointer = MutablePointer::new_from_ref(&mut value);
        // SAFETY: pointer is writeable and `value` and `pointer` are not concurrently used.
        unsafe {
            pointer.write(4321);
        }
        assert_eq!(4321, value);

        let mut pointer = pointer.clone();
        // SAFETY: pointer is writeable and `value` and `pointer` are not concurrently used.
        unsafe {
            pointer.write(2025);
        }
        assert_eq!(2025, value);
    }

    #[test]
    fn test_equality() {
        let pointer1 = ImmutablePointer::new(0x20008 as *const u64).unwrap();
        let pointer2 = ImmutablePointer::new(0x20008 as *const u64).unwrap();
        let pointer3 = ImmutablePointer::new(0x20010 as *const u64).unwrap();

        assert_eq!(pointer1, pointer2);
        assert_ne!(pointer1, pointer3);
        assert_ne!(pointer2, pointer3);
    }

    #[test]
    fn test_compare() {
        let pointer1 = ImmutablePointer::new(0x20008 as *const u64).unwrap();
        let pointer2 = ImmutablePointer::new(0x20010 as *const u64).unwrap();

        assert!(pointer1 < pointer2);
        assert!(pointer2 > pointer1);
    }

    #[test]
    fn test_distance_from_origin() {
        let origin = MutablePointer::new(0x1000 as *mut u32).unwrap();
        let pointer = MutablePointer::new(0x2000 as *mut u32).unwrap();

        // SAFETY: pointer > origin
        let distance = unsafe { pointer.distance_from_origin(&origin) };
        assert_eq!(0x400, distance.get());
    }

    #[test]
    fn test_offset_from() {
        let origin = ImmutablePointer::new(0x1000 as *mut u64).unwrap();
        let pointer = ImmutablePointer::new(0x2000 as *mut u64).unwrap();

        // SAFETY: let's assume the two pointers come from the same allocated object.
        let offset = unsafe { pointer.offset_from(&origin) };
        assert_eq!(0x200, offset);

        // SAFETY: let's assume the two pointers come from the same allocated object.
        let offset = unsafe { origin.offset_from(&pointer) };
        assert_eq!(-0x200, offset);
    }
}
