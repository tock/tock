// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2025.

//! Physical and virtual pointers.

use crate::utilities::alignment::{Alignment, AlwaysAligned};
use crate::utilities::ordering::SmallerPair;
use crate::utilities::pointers::{
    Error, ImmutablePointer as ImmutablePtr, MutablePointer as MutablePtr, Pointer as Ptr,
};

use core::cmp::Ordering;
use core::fmt::{Debug, Formatter};
use core::num::NonZero;
use core::ops::Sub;
use core::ptr::NonNull;

/// A pointer from the perspective of the memory management system.
#[repr(transparent)]
pub struct Pointer<const IS_VIRTUAL: bool, const IS_MUTABLE: bool, T: Alignment>(
    Ptr<IS_MUTABLE, T>,
);

/// An immutable pointer from the perspective of the memory management system.
pub type ImmutablePointer<const IS_VIRTUAL: bool, T> = Pointer<IS_VIRTUAL, false, T>;
/// A mutable pointer from the perspective of the memory management system.
pub type MutablePointer<const IS_VIRTUAL: bool, T> = Pointer<IS_VIRTUAL, true, T>;

/// A physical pointer.
pub type PhysicalPointer<const IS_MUTABLE: bool, T> = Pointer<false, IS_MUTABLE, T>;
/// An immutable physical pointer.
pub type ImmutablePhysicalPointer<T> = PhysicalPointer<false, T>;
/// A mutable physical pointer.
pub type MutablePhysicalPointer<T> = PhysicalPointer<true, T>;

/// A virtual pointer.
pub type VirtualPointer<const IS_MUTABLE: bool, T> = Pointer<true, IS_MUTABLE, T>;
/// An immutable virtual pointer.
pub type ImmutableVirtualPointer<T> = VirtualPointer<false, T>;
/// A mutable virtual pointer.
pub type MutableVirtualPointer<T> = VirtualPointer<true, T>;

/// A valid virtual pointer, that is, a virtual pointer that belongs to the kernel or to a process.
#[repr(transparent)]
pub struct ValidVirtualPointer<const IS_USER: bool, const IS_MUTABLE: bool, T: Alignment>(
    Pointer<true, IS_MUTABLE, T>,
);

/// A valid immutable virtual pointer.
pub type ValidImmutableVirtualPointer<const IS_USER: bool, T> =
    ValidVirtualPointer<IS_USER, false, T>;
/// A valid mutable virtual pointer.
pub type ValidMutableVirtualPointer<const IS_USER: bool, T> = ValidVirtualPointer<IS_USER, true, T>;

/// A user virtual pointer.
pub type UserVirtualPointer<const IS_MUTABLE: bool, T> = ValidVirtualPointer<true, IS_MUTABLE, T>;
/// An immutable user virtual pointer.
pub type ImmutableUserVirtualPointer<T> = UserVirtualPointer<false, T>;
/// A mutable user virtual pointer.
pub type MutableUserVirtualPointer<T> = UserVirtualPointer<true, T>;

/// A kernel virtual pointer.
pub type KernelVirtualPointer<const IS_MUTABLE: bool, T> =
    ValidVirtualPointer<false, IS_MUTABLE, T>;
/// An immutable kernel virtual pointer.
pub type ImmutableKernelVirtualPointer<T> = KernelVirtualPointer<false, T>;
/// A mutable kernel virtual pointer.
pub type MutableKernelVirtualPointer<T> = KernelVirtualPointer<true, T>;

/// A nullable pointer from the perspective of the memory management system.
#[repr(C)]
pub enum NullablePointer<const IS_VIRTUAL: bool, const IS_MUTABLE: bool, T: Alignment> {
    Null,
    NonNull(Pointer<IS_VIRTUAL, IS_MUTABLE, T>),
}

/// An immutable nullable pointer from the perspective of the memory management system.
pub type ImmutableNullablePointer<const IS_VIRTUAL: bool, T> =
    NullablePointer<IS_VIRTUAL, false, T>;
/// A mutable nullable pointer from the memory management system pe
pub type MutableNullablePointer<const IS_VIRTUAL: bool, T> = NullablePointer<IS_VIRTUAL, true, T>;

/// A nullable physical pointer.
pub type NullablePhysicalPointer<const IS_MUTABLE: bool, T> = NullablePointer<false, IS_MUTABLE, T>;
/// An immutable nullable physical pointer.
pub type ImmutableNullablePhysicalPointer<T> = NullablePhysicalPointer<false, T>;
/// A mutable nullable physical pointer.
pub type MutableNullablePhysicalPointer<T> = NullablePhysicalPointer<true, T>;

/// A nullable virtual pointer.
pub type NullableVirtualPointer<const IS_MUTABLE: bool, T> = NullablePointer<true, IS_MUTABLE, T>;
/// An immutable nullable virtual pointer.
pub type ImmutableNullableVirtualPointer<T> = NullableVirtualPointer<false, T>;
/// A mutable nullable virtual pointer.
pub type MutableNullableVirtualPointer<T> = NullableVirtualPointer<true, T>;

/// A valid nullable virtual pointer, that is, a virtual pointer that belongs to the kernel or to a
/// process.
#[repr(C)]
#[derive(Debug)]
pub enum ValidNullableVirtualPointer<const IS_USER: bool, const IS_MUTABLE: bool, T: Alignment> {
    Null,
    NonNull(ValidVirtualPointer<IS_USER, IS_MUTABLE, T>),
}

/// An immutable valid nullable virtual pointer.
pub type ImmutableValidNullableVirtualPointer<const IS_USER: bool, T> =
    ValidNullableVirtualPointer<IS_USER, false, T>;
/// A mutable valid nullable virtual pointer.
pub type MutableValidNullableVirtualPointer<const IS_USER: bool, T> =
    ValidNullableVirtualPointer<IS_USER, true, T>;

/// A user nullable virtual pointer.
pub type UserNullableVirtualPointer<const IS_MUTABLE: bool, T> =
    ValidNullableVirtualPointer<true, IS_MUTABLE, T>;
/// An immutable user nullable virtual pointer.
pub type ImmutableUserNullableVirtualPointer<T> = UserNullableVirtualPointer<false, T>;
/// A mutable user nullable virtual pointer.
pub type MutableUserNullableVirtualPointer<T> = UserNullableVirtualPointer<true, T>;

/// A kernel nullable virtual pointer.
pub type KernelNullableVirtualPointer<const IS_MUTABLE: bool, T> =
    ValidNullableVirtualPointer<false, IS_MUTABLE, T>;
/// An immutable kernel virtual pointer.
pub type ImmutableKernelNullableVirtualPointer<T> = KernelNullableVirtualPointer<false, T>;
/// A mutable kernel virtual pointer.
pub type MutableKernelNullableVirtualPointer<T> = KernelNullableVirtualPointer<true, T>;

impl<const IS_VIRTUAL: bool, const IS_MUTABLE: bool, T: Alignment>
    Pointer<IS_VIRTUAL, IS_MUTABLE, T>
{
    /// # Safety
    ///
    /// The caller must ensure that `pointer` is of right type.
    pub(crate) const unsafe fn new_ref(pointer: &Ptr<IS_MUTABLE, T>) -> &Self {
        // SAFETY: `Pointer` is marked #[repr(transparent)], so a pointer to `Ptr` is also a valid
        // pointer to `Pointer`
        unsafe { &*core::ptr::from_ref(pointer).cast() }
    }

    /// # Safety
    ///
    /// The caller must ensure that `pointer` is of right type.
    pub const unsafe fn new(pointer: Ptr<IS_MUTABLE, T>) -> Self {
        Self(pointer)
    }

    /// # Safety
    ///
    /// The caller must ensure that `non_null_pointer` is of right type.
    pub(crate) unsafe fn new_non_null(non_null_pointer: NonNull<T>) -> Result<Self, ()> {
        let aligned_non_null_pointer = match Ptr::new_non_null(non_null_pointer) {
            Err(()) => return Err(()),
            Ok(aligned_non_null_pointer) => aligned_non_null_pointer,
        };

        // SAFETY: the caller ensures that `non_null_pointer` is of right type.
        let pointer = unsafe { Self::new(aligned_non_null_pointer) };
        Ok(pointer)
    }

    const fn as_inner(&self) -> &Ptr<IS_MUTABLE, T> {
        &self.0
    }

    pub(super) const fn downgrade(self) -> Ptr<IS_MUTABLE, T> {
        self.0
    }

    pub const fn to_nullable(self) -> NullablePointer<IS_VIRTUAL, IS_MUTABLE, T> {
        NullablePointer::new_non_null(self)
    }

    pub fn get_address(&self) -> NonZero<usize> {
        self.as_inner().get_address()
    }

    pub const fn to_immutable(self) -> Pointer<IS_VIRTUAL, false, T> {
        let mutable_inner = self.downgrade();
        let immutable_inner = mutable_inner.to_immutable();
        // SAFETY: `immutable_inner` comes from self
        unsafe { Pointer::new(immutable_inner) }
    }

    pub const fn as_immutable(&self) -> &Pointer<IS_VIRTUAL, false, T> {
        // SAFETY: `Pointer` is marked #[repr(transparent)], so its memory layout is the same for
        // both immutable and mutable pointers
        unsafe { &*core::ptr::from_ref(self).cast() }
    }

    pub const fn infallible_cast<U: AlwaysAligned>(self) -> Pointer<IS_VIRTUAL, IS_MUTABLE, U> {
        // SAFETY: `Pointer` is marked #[repr(transparent)], so its memory layout is the same for
        // different types.
        unsafe { *core::ptr::from_ref(&self).cast() }
    }

    pub const fn infallible_cast_ref<U: AlwaysAligned>(
        &self,
    ) -> &Pointer<IS_VIRTUAL, IS_MUTABLE, U> {
        // SAFETY: `Pointer` is marked #[repr(transparent)], so its memory layout is the same for
        // different types.
        unsafe { &*core::ptr::from_ref(self).cast() }
    }

    /// # Safety
    ///
    /// The addition must not overflow.
    pub(crate) unsafe fn unchecked_add(&self, count: NonZero<usize>) -> Self {
        let inner = self.as_inner();
        let new_inner = inner.unchecked_add(count);
        let new_pointer = Self::new(new_inner);
        new_pointer
    }

    pub fn checked_add(&self, count: NonZero<usize>) -> Result<Self, ()> {
        let inner = self.as_inner();
        let new_inner = match inner.checked_add(count) {
            Err(()) => return Err(()),
            Ok(new_inner) => new_inner,
        };
        // SAFETY: `new_inner` comes from `self`
        let new_pointer = unsafe { Self::new(new_inner) };
        Ok(new_pointer)
    }

    pub fn checked_sub(&self, count: NonZero<usize>) -> Result<Self, ()> {
        let inner = self.as_inner();
        let new_inner = match inner.checked_sub(count) {
            Err(()) => return Err(()),
            Ok(new_inner) => new_inner,
        };
        // SAFETY: `new_inner` comes from `self`
        let new_pointer = unsafe { Self::new(new_inner) };
        Ok(new_pointer)
    }

    pub fn checked_offset(&self, count: NonZero<isize>) -> Result<Self, ()> {
        let inner = self.as_inner();
        let new_inner = match inner.checked_offset(count) {
            Err(()) => return Err(()),
            Ok(new_inner) => new_inner,
        };
        // SAFETY: `new_inner` comes from `self`
        let new_pointer = unsafe { Self::new(new_inner) };
        Ok(new_pointer)
    }

    /// # Safety
    ///
    /// The addition must not overflow.
    pub(crate) unsafe fn unchecked_add_zero(&self, count: usize) -> Self {
        match NonZero::new(count) {
            None => *self,
            // SAFETY: the caller ensures that the addition does not overflow
            Some(non_zero_count) => unsafe { self.unchecked_add(non_zero_count) },
        }
    }

    pub fn checked_add_zero(&self, count: usize) -> Result<Self, ()> {
        match NonZero::new(count) {
            None => Ok(*self),
            Some(non_zero_count) => self.checked_add(non_zero_count),
        }
    }

    pub fn checked_sub_zero(&self, count: usize) -> Result<Self, ()> {
        match NonZero::new(count) {
            None => Ok(*self),
            Some(non_zero_count) => self.checked_add(non_zero_count),
        }
    }

    pub fn checked_offset_zero(&self, count: isize) -> Result<Self, ()> {
        match NonZero::new(count) {
            None => Ok(*self),
            Some(non_zero_count) => self.checked_offset(non_zero_count),
        }
    }

    /// # Safety
    ///
    /// Same safety requirements as `core::ptr::sub_ptr()` + `self` > `origin`
    pub unsafe fn distance_from_origin(&self, origin: &Self) -> NonZero<usize> {
        let inner = self.as_inner();
        let origin_inner = origin.as_inner();

        // SAFETY: the caller ensures that `self` > `origin`
        unsafe { inner.distance_from_origin(origin_inner) }
    }

    /// # Safety
    ///
    /// Same safety requirements as `core::ptr::sub_ptr()`
    pub const unsafe fn offset_from(&self, origin: &Self) -> isize {
        let inner = self.as_inner();
        let origin_inner = origin.as_inner();

        // SAFETY: the caller ensures that `self` > `origin`
        unsafe { inner.offset_from(origin_inner) }
    }

    pub fn cast<U: Alignment>(self) -> Result<Pointer<IS_VIRTUAL, IS_MUTABLE, U>, Self> {
        let inner = self.downgrade();
        let casted_inner = match inner.cast() {
            // SAFETY: `inner` comes from `self`
            Err(inner) => return Err(unsafe { Self::new(inner) }),
            Ok(casted_inner) => casted_inner,
        };

        // SAFETY: `casted_inner` comes from `self`
        Ok(unsafe { Pointer::new(casted_inner) })
    }
}

impl<const IS_VIRTUAL: bool, const IS_MUTABLE: bool, U: AlwaysAligned>
    Pointer<IS_VIRTUAL, IS_MUTABLE, U>
{
    /// # Safety
    ///
    /// The caller must ensure that `non_null_pointer` is of right type.
    pub(crate) unsafe fn new_from_non_null_byte(non_null_pointer: NonNull<U>) -> Self {
        let aligned_non_null_pointer = Ptr::new_from_non_null_byte(non_null_pointer);

        // SAFETY: the caller ensures that `non_null_pointer` is of right type.
        unsafe { Self::new(aligned_non_null_pointer) }
    }
}

impl<const IS_VIRTUAL: bool, const IS_MUTABLE: bool, T: Alignment> core::fmt::LowerHex
    for Pointer<IS_VIRTUAL, IS_MUTABLE, T>
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "{:#x}", self.get_address())
    }
}

impl<const IS_MUTABLE: bool, T: Alignment> PhysicalPointer<IS_MUTABLE, T> {
    pub const fn to_virtual_pointer(self) -> VirtualPointer<IS_MUTABLE, T> {
        // SAFETY: the duplicated pointer is converted immediately to a virtual pointer, so there
        // will be no two pointers to the same physical memory.
        let inner = self.downgrade();
        // SAFETY: the flat mapping of a physical pointer is always a valid virtual pointer
        unsafe { VirtualPointer::new(inner) }
    }

    pub(crate) const fn to_valid_virtual_pointer<const IS_USER: bool>(
        self,
    ) -> ValidVirtualPointer<IS_USER, IS_MUTABLE, T> {
        let virtual_pointer = self.to_virtual_pointer();
        // SAFETY: the flat mapping of a physical pointer is always a valid user virtual pointer
        unsafe { ValidVirtualPointer::new(virtual_pointer) }
    }
}

impl<const IS_MUTABLE: bool, T: Alignment> VirtualPointer<IS_MUTABLE, T> {
    /// # SAFETY
    ///
    /// The caller must ensure that the flat mapping of `self` is a valid physical pointer.
    pub const unsafe fn to_physical_pointer(self) -> PhysicalPointer<IS_MUTABLE, T> {
        let inner = self.downgrade();
        // SAFETY: The caller ensures that the flat mapping of this pointer is a valid physical
        // pointer.
        unsafe { PhysicalPointer::new(inner) }
    }
}

impl<const IS_VIRTUAL: bool, T: Alignment> ImmutablePointer<IS_VIRTUAL, T> {
    /// # Safety
    ///
    /// The caller must ensure that `pointer` is of right type.
    pub(crate) unsafe fn new_raw(pointer: *const T) -> Result<Self, Error> {
        let pointer = ImmutablePtr::new(pointer)?;

        // SAFETY: The caller ensures that `pointer` is of the right type
        let pointer = unsafe { Self::new(pointer) };

        Ok(pointer)
    }

    /// # Safety
    ///
    /// The caller must ensure that `reference` is of right type.
    pub unsafe fn new_from_ref(reference: &T) -> Self {
        let immutable_pointer = ImmutablePtr::new_from_ref(reference);
        // SAFETY: The caller ensures that `reference` is of the right type
        unsafe { Self::new(immutable_pointer) }
    }

    /// # Safety
    ///
    /// The two pointers must not be used at the same time.
    pub(crate) const unsafe fn as_raw(&self) -> *const T {
        self.as_inner().as_raw()
    }

    /// # Safety
    ///
    /// The two pointers must not be used at the same time.
    const unsafe fn to_raw(self) -> *const T {
        self.as_inner().to_raw()
    }
}

impl<const IS_VIRTUAL: bool, U: AlwaysAligned> ImmutablePointer<IS_VIRTUAL, U> {
    /// # Safety
    ///
    /// The caller must ensure that `pointer` is either:
    ///
    /// 1. virtual if IS_VIRTUAL == true
    /// 2. physical if IS_VIRTUAL == false
    pub(crate) unsafe fn new_from_raw_byte(pointer: *const U) -> Result<Self, ()> {
        let non_null_pointer = match NonNull::new(pointer.cast_mut()) {
            None => return Err(()),
            Some(non_null_pointer) => non_null_pointer,
        };

        let pointer = Ptr::new_from_non_null_byte(non_null_pointer);

        Ok(Self(pointer))
    }
}

impl<const IS_VIRTUAL: bool, T: Alignment> MutablePointer<IS_VIRTUAL, T> {
    /// # Safety
    ///
    /// The caller must ensure that `pointer` is of right type.
    pub unsafe fn new_raw(pointer: *mut T) -> Result<Self, Error> {
        let pointer = MutablePtr::new(pointer)?;

        // SAFETY: The caller ensures that `pointer` is of the right type
        let pointer = unsafe { Self::new(pointer) };

        Ok(pointer)
    }

    /// # Safety
    ///
    /// The caller must ensure that the two pointers are not used at the same time.
    pub const unsafe fn to_raw(self) -> *mut T {
        self.downgrade().to_raw()
    }

    /// # Safety
    ///
    /// The two pointers must not be used at the same time.
    pub(crate) const unsafe fn as_raw(&self) -> *mut T {
        self.as_inner().as_raw()
    }

    pub fn cast_mutability<const IS_MUTABLE: bool>(self) -> Pointer<IS_VIRTUAL, IS_MUTABLE, T> {
        // SAFETY: TODO
        unsafe {
            core::mem::transmute::<MutablePointer<IS_VIRTUAL, T>, Pointer<IS_VIRTUAL, IS_MUTABLE, T>>(
                self,
            )
        }
    }

    /// # Safety
    ///
    /// `self` must be valid for writes and no reference to the memory pointed by `self` must exist
    pub(crate) unsafe fn write(&mut self, value: T) {
        // SAFETY:
        //
        // 1. `self` is suitably aligned.
        // 2. `self` is not null.
        // 3. the method's precondition ensures that `self` points to writeable memory.
        // 4. the method's precondition ensures that no reference to the memory pointed by `self`
        //    exists.
        self.0.write(value)
    }
}

impl<const IS_VIRTUAL: bool, const IS_MUTABLE: bool, T: Alignment> PartialEq
    for Pointer<IS_VIRTUAL, IS_MUTABLE, T>
{
    fn eq(&self, other: &Self) -> bool {
        self.as_inner() == other.as_inner()
    }
}

impl<const IS_VIRTUAL: bool, const IS_MUTABLE: bool, T: Alignment> Eq
    for Pointer<IS_VIRTUAL, IS_MUTABLE, T>
{
}

impl<const IS_VIRTUAL: bool, const IS_MUTABLE: bool, T: Alignment> PartialOrd
    for Pointer<IS_VIRTUAL, IS_MUTABLE, T>
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<const IS_VIRTUAL: bool, const IS_MUTABLE: bool, T: Alignment> Ord
    for Pointer<IS_VIRTUAL, IS_MUTABLE, T>
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_inner().cmp(other.as_inner())
    }
}

impl<const IS_VIRTUAL: bool, const IS_MUTABLE: bool, T: Alignment> Sub
    for &Pointer<IS_VIRTUAL, IS_MUTABLE, T>
{
    type Output = isize;

    fn sub(self, other: Self) -> Self::Output {
        self.as_inner().sub(other.as_inner())
    }
}

impl<const IS_VIRTUAL: bool, const IS_MUTABLE: bool, T: Alignment> Debug
    for Pointer<IS_VIRTUAL, IS_MUTABLE, T>
{
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "{:?}", self.as_inner())
    }
}

impl<const IS_VIRTUAL: bool, const IS_MUTABLE: bool, T: Alignment> Clone
    for Pointer<IS_VIRTUAL, IS_MUTABLE, T>
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<const IS_VIRTUAL: bool, const IS_MUTABLE: bool, T: Alignment> Copy
    for Pointer<IS_VIRTUAL, IS_MUTABLE, T>
{
}

impl<const IS_USER: bool, const IS_MUTABLE: bool, T: Alignment>
    ValidVirtualPointer<IS_USER, IS_MUTABLE, T>
{
    /// # Safety
    ///
    /// The caller must ensure that `virtual_pointer` is a valid:
    ///
    /// 1. User virtual pointer if IS_USER == true
    /// 2. Kernel virtual pointer if IS_USER == false
    pub(crate) const unsafe fn new(virtual_pointer: VirtualPointer<IS_MUTABLE, T>) -> Self {
        Self(virtual_pointer)
    }

    pub(super) const fn as_virtual_pointer(&self) -> &VirtualPointer<IS_MUTABLE, T> {
        &self.0
    }

    pub const fn to_virtual_pointer(&self) -> VirtualPointer<IS_MUTABLE, T> {
        self.0
    }

    pub(crate) fn as_immutable(&self) -> &ValidImmutableVirtualPointer<IS_USER, T> {
        // SAFETY: `ValidVirtualPointer` is marked #[repr(transparent)], so it has the same memory
        // representation for both immutable and mutable counter parts.
        unsafe { &*core::ptr::from_ref(self).cast() }
    }

    pub(crate) fn to_immutable(self) -> ValidImmutableVirtualPointer<IS_USER, T> {
        // SAFETY: `ValidVirtualPointer` is marked #[repr(transparent)], so it has the same memory
        // representation for both immutable and mutable counter parts.
        unsafe { *core::ptr::from_ref(&self).cast() }
    }

    pub fn to_nullable(self) -> ValidNullableVirtualPointer<IS_USER, IS_MUTABLE, T> {
        ValidNullableVirtualPointer::new_non_null(self)
    }

    pub(super) const fn downgrade(self) -> VirtualPointer<IS_MUTABLE, T> {
        self.0
    }

    pub const fn infallible_cast<U: AlwaysAligned>(
        self,
    ) -> ValidVirtualPointer<IS_USER, IS_MUTABLE, U> {
        // SAFETY: `ValidVirtualPointer` is marked #[repr(transparent)], so its memory layout is the same for
        // different types.
        unsafe { *core::ptr::from_ref(&self).cast() }
    }

    pub const fn infallible_cast_ref<U: AlwaysAligned>(
        &self,
    ) -> &ValidVirtualPointer<IS_USER, IS_MUTABLE, U> {
        // SAFETY: `ValidVirtualPointer` is marked #[repr(transparent)], so its memory layout is the same for
        // different types.
        unsafe { &*core::ptr::from_ref(self).cast() }
    }

    pub fn get_address(&self) -> NonZero<usize> {
        self.as_virtual_pointer().get_address()
    }

    /// # Safety
    ///
    /// The addition must not overflow.
    pub(crate) unsafe fn unchecked_add(&self, count: NonZero<usize>) -> Self {
        let virtual_pointer = self.as_virtual_pointer();
        // SAFETY: The caller ensures that the addition does not overflow.
        let new_virtual_pointer = unsafe { virtual_pointer.unchecked_add(count) };
        // SAFETY: `new_virtual_pointer` comes from `self`
        let new_pointer = unsafe { Self::new(new_virtual_pointer) };
        new_pointer
    }

    pub(crate) fn checked_add(&self, count: NonZero<usize>) -> Result<Self, ()> {
        let virtual_pointer = self.as_virtual_pointer().checked_add(count)?;
        // SAFETY: `virtual_pointer` comes from `self`
        let new_pointer = unsafe { Self::new(virtual_pointer) };
        Ok(new_pointer)
    }

    /// # Safety
    ///
    /// The addition must not overflow.
    pub(crate) unsafe fn unchecked_add_zero(&self, count: usize) -> Self {
        match NonZero::new(count) {
            None => *self,
            // SAFETY: the caller ensures that the addition does not overflow
            Some(non_zero_count) => unsafe { self.unchecked_add(non_zero_count) },
        }
    }

    pub fn checked_offset(&self, count: NonZero<isize>) -> Result<Self, ()> {
        let virtual_pointer = self.to_virtual_pointer();
        let new_virtual_pointer = match virtual_pointer.checked_offset(count) {
            Err(()) => return Err(()),
            Ok(new_virtual_pointer) => new_virtual_pointer,
        };
        // SAFETY: `new_virtual_pointer` comes from `self`
        let new_pointer = unsafe { Self::new(new_virtual_pointer) };
        Ok(new_pointer)
    }

    /// # Safety
    ///
    /// Same safety requirements as `core::ptr::sub_ptr()`
    pub const unsafe fn offset_from(&self, origin: &Self) -> isize {
        let inner = self.as_virtual_pointer();
        let origin_inner = origin.as_virtual_pointer();

        // SAFETY: the caller ensures that `self` > `origin`
        unsafe { inner.offset_from(origin_inner) }
    }

    pub fn cast<U: Alignment>(self) -> Result<ValidVirtualPointer<IS_USER, IS_MUTABLE, U>, Self> {
        let inner = self.downgrade();
        let casted_inner = match inner.cast() {
            // SAFETY: `inner` comes from `self`
            Err(inner) => return Err(unsafe { Self::new(inner) }),
            Ok(casted_inner) => casted_inner,
        };

        // SAFETY: `casted_inner` comes from `self`
        Ok(unsafe { ValidVirtualPointer::new(casted_inner) })
    }
}

impl<const IS_USER: bool, const IS_MUTABLE: bool, U: AlwaysAligned>
    ValidVirtualPointer<IS_USER, IS_MUTABLE, U>
{
    /// # Safety
    ///
    /// The caller must ensure that `non_null_pointer` is a virtual pointer. It must also ensure
    /// that it is:
    ///
    /// 1. User virtual pointer if IS_USER == true
    /// 2. Kernel virtual pointer if IS_USER == false
    pub(crate) unsafe fn new_from_non_null_byte(non_null_pointer: NonNull<U>) -> Self {
        // SAFETY: the caller ensures that `non_null_pointer` is a virtual pointer
        let virtual_pointer = unsafe { VirtualPointer::new_from_non_null_byte(non_null_pointer) };
        // SAFETY: the caller ensures that `non_null_pointer` is of the right type
        unsafe { Self::new(virtual_pointer) }
    }
}

impl<const IS_USER: bool, const IS_MUTABLE: bool, T: Alignment> core::fmt::LowerHex
    for ValidVirtualPointer<IS_USER, IS_MUTABLE, T>
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "{:#x}", self.get_address())
    }
}

impl<const IS_USER: bool, T: Alignment> ValidImmutableVirtualPointer<IS_USER, T> {
    /// # Safety
    ///
    /// The caller must ensure that `pointer` is a valid:
    ///
    /// 1. User virtual pointer if IS_USER == true
    /// 2. Kernel virtual pointer if IS_USER == false
    pub unsafe fn new_from_raw(pointer: *const T) -> Result<Self, Error> {
        // SAFETY: the caller ensures that `pointer` is a virtual pointer.
        let virtual_pointer = unsafe { ImmutableVirtualPointer::new_raw(pointer)? };
        // SAFETY: the caller ensures that `pointer` is of right type.
        let valid_virtual_pointer = unsafe { ValidImmutableVirtualPointer::new(virtual_pointer) };
        Ok(valid_virtual_pointer)
    }

    /// # Safety
    ///
    ///
    /// The caller must ensure that `reference` is a valid:
    ///
    /// 1. User virtual reference if IS_USER == true
    /// 2. Kernel virtual reference if IS_USER == false
    pub unsafe fn new_from_ref(reference: &T) -> Self {
        let virtual_pointer = ImmutableVirtualPointer::new_from_ref(reference);
        // SAFETY: The caller ensures that `reference` is of the right type
        unsafe { Self::new(virtual_pointer) }
    }

    /// # Safety
    ///
    /// The caller must ensure that the two pointers are not used at the same time.
    pub unsafe fn to_raw(self) -> *const T {
        let virtual_pointer = self.as_virtual_pointer();
        unsafe { virtual_pointer.to_raw() }
    }

    /// # Safety
    ///
    /// The caller must ensure that the two pointers are not used at the same time.
    pub(crate) unsafe fn as_raw(&self) -> *const T {
        let virtual_pointer = self.as_virtual_pointer();
        // SAFETY: the caller ensures that the two pointers are not used at the same time.
        unsafe { virtual_pointer.as_raw() }
    }
}

impl<const IS_USER: bool, U: AlwaysAligned> ValidImmutableVirtualPointer<IS_USER, U> {
    /// # Safety
    ///
    /// The caller must ensure that `pointer` is a virtual pointer. It must also ensure
    /// that it is:
    ///
    /// 1. User virtual pointer if IS_USER == true
    /// 2. Kernel virtual pointer if IS_USER == false
    pub(crate) unsafe fn new_from_raw_byte(pointer: *const U) -> Result<Self, ()> {
        let non_null_pointer = match NonNull::new(pointer.cast_mut()) {
            None => return Err(()),
            Some(non_null_pointer) => non_null_pointer,
        };

        // SAFETY: the caller ensures that `pointer` is of the right type.
        let virtual_pointer = unsafe { Self::new_from_non_null_byte(non_null_pointer) };

        Ok(virtual_pointer)
    }
}

impl<const IS_USER: bool, T: Alignment> ValidMutableVirtualPointer<IS_USER, T> {
    /// # Safety
    ///
    /// The caller must ensure that `pointer` is a valid:
    ///
    /// 1. User virtual pointer if IS_USER == true
    /// 2. Kernel virtual pointer if IS_USER == false
    pub unsafe fn new_from_raw(pointer: *mut T) -> Result<Self, Error> {
        // SAFETY: the caller ensures that `pointer` is a virtual pointer.
        let virtual_pointer = unsafe { MutableVirtualPointer::new_raw(pointer)? };
        // SAFETY: the caller ensures that `pointer` is of right type.
        let valid_virtual_pointer = unsafe { ValidMutableVirtualPointer::new(virtual_pointer) };
        Ok(valid_virtual_pointer)
    }

    /// # Safety
    ///
    /// The caller must ensure that the two pointers are not used at the same time.
    pub(crate) unsafe fn as_raw(&self) -> *mut T {
        let virtual_pointer = self.as_virtual_pointer();
        // SAFETY: the caller must ensure that the two pointers are not used at the same time.
        unsafe { virtual_pointer.as_raw() }
    }

    /// # Safety
    ///
    /// The caller must ensure that the two pointers are not used at the same time.
    pub(crate) unsafe fn to_raw(self) -> *mut T {
        let virtual_pointer = self.to_virtual_pointer();
        // SAFETY: the caller must ensure that the two pointers are not used at the same time.
        unsafe { virtual_pointer.to_raw() }
    }

    pub fn cast_mutability<const IS_MUTABLE: bool>(
        self,
    ) -> ValidVirtualPointer<IS_USER, IS_MUTABLE, T> {
        // SAFETY: TODO
        unsafe {
            core::mem::transmute::<
                ValidMutableVirtualPointer<IS_USER, T>,
                ValidVirtualPointer<IS_USER, IS_MUTABLE, T>,
            >(self)
        }
    }

    /// # Safety
    ///
    /// The caller must ensure that no other pointer or reference exists to the memory pointed by
    /// `self`.
    pub(crate) unsafe fn write(&mut self, value: T) {
        unsafe { self.0.write(value) }
    }
}

impl<const IS_USER: bool, U: AlwaysAligned> ValidMutableVirtualPointer<IS_USER, U> {
    /// # Safety
    ///
    /// The caller must ensure that `pointer` is a virtual pointer. It must also ensure
    /// that it is:
    ///
    /// 1. User virtual pointer if IS_USER == true
    /// 2. Kernel virtual pointer if IS_USER == false
    pub(crate) unsafe fn new_from_raw_byte(pointer: *const U) -> Result<Self, ()> {
        let non_null_pointer = match NonNull::new(pointer.cast_mut()) {
            None => return Err(()),
            Some(non_null_pointer) => non_null_pointer,
        };

        // SAFETY: the caller ensures that `pointer` is of the right type.
        let virtual_pointer = unsafe { Self::new_from_non_null_byte(non_null_pointer) };

        Ok(virtual_pointer)
    }
}

impl<const IS_USER: bool, const IS_MUTABLE: bool, T: Alignment> PartialEq
    for ValidVirtualPointer<IS_USER, IS_MUTABLE, T>
{
    fn eq(&self, other: &Self) -> bool {
        self.as_virtual_pointer().eq(other.as_virtual_pointer())
    }
}

impl<const IS_USER: bool, const IS_MUTABLE: bool, T: Alignment> Eq
    for ValidVirtualPointer<IS_USER, IS_MUTABLE, T>
{
}

impl<const IS_USER: bool, const IS_MUTABLE: bool, T: Alignment> PartialOrd
    for ValidVirtualPointer<IS_USER, IS_MUTABLE, T>
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<const IS_USER: bool, const IS_MUTABLE: bool, T: Alignment> Ord
    for ValidVirtualPointer<IS_USER, IS_MUTABLE, T>
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_virtual_pointer().cmp(other.as_virtual_pointer())
    }
}

impl<const IS_USER: bool, const IS_MUTABLE: bool, T: Alignment> Sub
    for &ValidVirtualPointer<IS_USER, IS_MUTABLE, T>
{
    type Output = isize;

    fn sub(self, other: Self) -> Self::Output {
        self.as_virtual_pointer().sub(other.as_virtual_pointer())
    }
}

impl<const IS_USER: bool, const IS_MUTABLE: bool, T: Alignment> Debug
    for ValidVirtualPointer<IS_USER, IS_MUTABLE, T>
{
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "{:?}", self.as_virtual_pointer())
    }
}

impl<const IS_USER: bool, const IS_MUTABLE: bool, T: Alignment> Clone
    for ValidVirtualPointer<IS_USER, IS_MUTABLE, T>
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<const IS_USER: bool, const IS_MUTABLE: bool, T: Alignment> Copy
    for ValidVirtualPointer<IS_USER, IS_MUTABLE, T>
{
}

impl<const IS_VIRTUAL: bool, const IS_MUTABLE: bool, T: Alignment>
    NullablePointer<IS_VIRTUAL, IS_MUTABLE, T>
{
    pub const fn new_null() -> Self {
        NullablePointer::Null
    }

    pub const fn new_non_null(pointer: Pointer<IS_VIRTUAL, IS_MUTABLE, T>) -> Self {
        NullablePointer::NonNull(pointer)
    }

    pub const fn infallible_cast<U: AlwaysAligned>(
        self,
    ) -> NullablePointer<IS_VIRTUAL, IS_MUTABLE, U> {
        match self {
            NullablePointer::Null => NullablePointer::new_null(),
            NullablePointer::NonNull(pointer) => {
                NullablePointer::new_non_null(pointer.infallible_cast())
            }
        }
    }

    pub fn get_address(&self) -> usize {
        match self {
            NullablePointer::Null => 0,
            NullablePointer::NonNull(pointer) => pointer.get_address().get(),
        }
    }

    pub fn checked_add(
        self,
        count: NonZero<usize>,
    ) -> Result<Pointer<IS_VIRTUAL, IS_MUTABLE, T>, ()> {
        let address = self.get_address();

        let byte_count = match count.get().checked_mul(core::mem::size_of::<T>()) {
            None => return Err(()),
            Some(byte_count) => byte_count,
        };

        let new_address = match address.checked_add(byte_count) {
            None => return Err(()),
            Some(new_address) => new_address,
        };

        // SAFETY: adding size_of::<T> * count bytes to a T-aligned pointer without overflow
        // results in a non-null T-aligned pointer
        let non_null_pointer = unsafe { NonNull::new_unchecked(new_address as *mut _) };

        // SAFETY: `self` guarantees that `non_null_pointer` has the right type.
        unsafe { Pointer::new_non_null(non_null_pointer) }
    }
}

impl<const IS_MUTABLE: bool, T: Alignment> NullableVirtualPointer<IS_MUTABLE, T> {
    /// # Safety
    ///
    /// The caller must ensure that the flat mapping of `self` is a valid physical pointer.
    pub const unsafe fn to_physical_pointer(self) -> NullablePhysicalPointer<IS_MUTABLE, T> {
        match self {
            NullableVirtualPointer::Null => NullablePhysicalPointer::Null,
            NullableVirtualPointer::NonNull(non_null_pointer) =>
            // SAFETY: The caller ensures that the flat mapping is valid.
            {
                NullablePhysicalPointer::NonNull(unsafe { non_null_pointer.to_physical_pointer() })
            }
        }
    }
}

impl<T: Alignment> ImmutableNullableVirtualPointer<T> {
    /// # Safety
    ///
    /// The caller must ensure that the two pointers are not used at the same time.
    pub unsafe fn to_raw(self) -> *const T {
        match self {
            ImmutableNullableVirtualPointer::Null => core::ptr::null(),
            // SAFETY: the caller ensures that the two pointers are not used at the same time.
            ImmutableNullableVirtualPointer::NonNull(non_null_pointer) => unsafe {
                non_null_pointer.to_raw()
            },
        }
    }
}

impl<T: Alignment> MutableNullableVirtualPointer<T> {
    /// # Safety
    ///
    /// The caller must ensure that the two pointers are not used at the same time.
    pub unsafe fn to_raw(self) -> *mut T {
        match self {
            MutableNullableVirtualPointer::Null => core::ptr::null_mut(),
            // SAFETY: the caller ensures that the two pointers are not used at the same time.
            MutableNullableVirtualPointer::NonNull(non_null_pointer) => unsafe {
                non_null_pointer.to_raw()
            },
        }
    }
}

impl<const IS_VIRTUAL: bool, const IS_MUTABLE: bool, T: Alignment> Clone
    for NullablePointer<IS_VIRTUAL, IS_MUTABLE, T>
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<const IS_VIRTUAL: bool, const IS_MUTABLE: bool, T: Alignment> Copy
    for NullablePointer<IS_VIRTUAL, IS_MUTABLE, T>
{
}

impl<const IS_USER: bool, const IS_MUTABLE: bool, T: Alignment>
    ValidNullableVirtualPointer<IS_USER, IS_MUTABLE, T>
{
    pub const fn new_null() -> Self {
        ValidNullableVirtualPointer::Null
    }

    pub const fn new_non_null(
        valid_virtual_pointer: ValidVirtualPointer<IS_USER, IS_MUTABLE, T>,
    ) -> Self {
        ValidNullableVirtualPointer::NonNull(valid_virtual_pointer)
    }

    pub const fn downgrade(self) -> NullableVirtualPointer<IS_MUTABLE, T> {
        match self {
            ValidNullableVirtualPointer::Null => NullableVirtualPointer::new_null(),
            ValidNullableVirtualPointer::NonNull(valid_virtual_pointer) => {
                let virtual_pointer = valid_virtual_pointer.downgrade();
                NullableVirtualPointer::new_non_null(virtual_pointer)
            }
        }
    }

    pub fn get_address(&self) -> usize {
        match self {
            ValidNullableVirtualPointer::Null => 0,
            ValidNullableVirtualPointer::NonNull(valid_virtual_pointer) => {
                valid_virtual_pointer.get_address().get()
            }
        }
    }
}

impl<const IS_USER: bool, T: Alignment> ImmutableValidNullableVirtualPointer<IS_USER, T> {
    /// # Safety
    ///
    /// The caller must ensure that the two pointers are not used at the same time.
    pub unsafe fn to_raw(self) -> *const T {
        let nullable_virtual_pointer = self.downgrade();
        unsafe { nullable_virtual_pointer.to_raw() }
    }
}

impl<const IS_USER: bool, U: AlwaysAligned> ImmutableValidNullableVirtualPointer<IS_USER, U> {
    /// # Safety
    ///
    /// The caller must ensure that `pointer` is a valid user virtual pointer.
    pub unsafe fn new_from_byte(pointer: *const U) -> Self {
        match NonNull::new(pointer as *mut U) {
            None => ImmutableValidNullableVirtualPointer::Null,
            Some(non_null_pointer) => ImmutableValidNullableVirtualPointer::NonNull(
                ValidImmutableVirtualPointer::new_from_non_null_byte(non_null_pointer),
            ),
        }
    }
}

impl<const IS_USER: bool, T: Alignment> MutableValidNullableVirtualPointer<IS_USER, T> {
    /// # Safety
    ///
    /// The caller must ensure that the two pointers are not used at the same time.
    pub unsafe fn to_raw(self) -> *mut T {
        let nullable_virtual_pointer = self.downgrade();
        unsafe { nullable_virtual_pointer.to_raw() }
    }

    pub fn as_immutable(&self) -> &ImmutableValidNullableVirtualPointer<IS_USER, T> {
        // SAFETY: `ValidNullableVirtualPointer` is marked #[repr(C)], so its memory layout is the same for
        // both immutable and mutable pointers
        unsafe { &*core::ptr::from_ref(self).cast() }
    }
}

impl<const IS_USER: bool, U: AlwaysAligned> MutableValidNullableVirtualPointer<IS_USER, U> {
    /// # Safety
    ///
    /// The caller must ensure that `pointer` is a valid user virtual pointer.
    pub unsafe fn new_from_byte(pointer: *mut U) -> Self {
        match NonNull::new(pointer) {
            None => MutableValidNullableVirtualPointer::Null,
            Some(non_null_pointer) => MutableValidNullableVirtualPointer::NonNull(
                ValidMutableVirtualPointer::new_from_non_null_byte(non_null_pointer),
            ),
        }
    }
}

impl<const IS_USER: bool, const IS_MUTABLE: bool, T: Alignment> Default
    for ValidNullableVirtualPointer<IS_USER, IS_MUTABLE, T>
{
    fn default() -> Self {
        Self::new_null()
    }
}

impl<const IS_USER: bool, const IS_MUTABLE: bool, T: Alignment> Clone
    for ValidNullableVirtualPointer<IS_USER, IS_MUTABLE, T>
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<const IS_USER: bool, const IS_MUTABLE: bool, T: Alignment> Copy
    for ValidNullableVirtualPointer<IS_USER, IS_MUTABLE, T>
{
}

impl<const IS_USER: bool, T: Alignment> PartialEq
    for ImmutableValidNullableVirtualPointer<IS_USER, T>
{
    fn eq(&self, other: &Self) -> bool {
        // SAFETY: `raw_pointer` is used for comparaison, then discarded.
        let raw_pointer = unsafe { self.to_raw() };
        // SAFETY: `other_raw_pointer` is used for comparaison, then discarded.
        let other_raw_pointer = unsafe { other.to_raw() };

        raw_pointer == other_raw_pointer
    }
}

impl<const IS_USER: bool, T: Alignment> Eq for ImmutableValidNullableVirtualPointer<IS_USER, T> {}

impl<const IS_USER: bool, T: Alignment> PartialEq
    for MutableValidNullableVirtualPointer<IS_USER, T>
{
    fn eq(&self, other: &Self) -> bool {
        // SAFETY: `raw_pointer` is used for comparaison, then discarded.
        let raw_pointer = unsafe { self.to_raw() };
        // SAFETY: `other_raw_pointer` is used for comparaison, then discarded.
        let other_raw_pointer = unsafe { other.to_raw() };

        raw_pointer == other_raw_pointer
    }
}

impl<const IS_USER: bool, T: Alignment> Eq for MutableValidNullableVirtualPointer<IS_USER, T> {}

impl<const IS_VIRTUAL: bool, const IS_MUTABLE: bool, T: Alignment>
    SmallerPair<Pointer<IS_VIRTUAL, IS_MUTABLE, T>>
{
    pub(super) fn downgrade(self) -> SmallerPair<Ptr<IS_MUTABLE, T>> {
        let (smaller, bigger) = self.consume();
        let smaller = smaller.downgrade();
        let bigger = bigger.downgrade();
        // SAFETY: downgrade doesn't affect ordering
        unsafe { SmallerPair::new_unchecked(smaller, bigger) }
    }
}
