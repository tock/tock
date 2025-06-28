// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2025.

//! Regions of memory.

use super::permissions::Permissions;
use super::pointers::{
    ImmutablePhysicalPointer, ImmutablePointer, ImmutableVirtualPointer, MutablePhysicalPointer,
    MutablePointer, PhysicalPointer, Pointer, ValidImmutableVirtualPointer,
    ValidMutableVirtualPointer, ValidVirtualPointer,
};
use super::slices::{MutablePhysicalSlice, Slice};

use crate::utilities::alignment::{Alignment, AlwaysAligned};
use crate::utilities::misc::create_non_zero_usize;
use crate::utilities::ordering::{
    SmallerOrEqualPair, SmallerOrEqualPairImmutableReference, SmallerPair,
};

use core::cell::Cell;
use core::marker::PhantomData;
use core::num::NonZero;

/// Pointers representing the start and the end of a region.
type Pointers<const IS_VIRTUAL: bool, const IS_MUTABLE: bool, T> =
    SmallerPair<Pointer<IS_VIRTUAL, IS_MUTABLE, T>>;

impl<const IS_VIRTUAL: bool, const IS_MUTABLE: bool, T> Pointers<IS_VIRTUAL, IS_MUTABLE, T> {
    fn as_immutable(&self) -> &SmallerPair<Pointer<IS_VIRTUAL, false, T>> {
        // SAFETY: SmallerPair is marked #[repr(transparent)]
        unsafe { &*core::ptr::from_ref(self).cast() }
    }
}

/// A memory region.
pub struct Region<const IS_VIRTUAL: bool, const IS_MUTABLE: bool, T: Alignment>(
    Pointers<IS_VIRTUAL, IS_MUTABLE, T>,
);

/// A region of immutable memory.
pub type ImmutableRegion<const IS_VIRTUAL: bool, T> = Region<IS_VIRTUAL, false, T>;
/// A region of mutable memory.
pub type MutableRegion<const IS_VIRTUAL: bool, T> = Region<IS_VIRTUAL, true, T>;

/// A region of physical memory.
pub type PhysicalRegion<const IS_MUTABLE: bool, T> = Region<false, IS_MUTABLE, T>;
/// A region of immutable physical memory.
pub type ImmutablePhysicalRegion<T> = PhysicalRegion<false, T>;
/// A region of mutable physical memory.
pub type MutablePhysicalRegion<T> = PhysicalRegion<true, T>;

/// A region of virtual memory.
pub type VirtualRegion<const IS_MUTABLE: bool, T> = Region<true, IS_MUTABLE, T>;
/// A region of immutable virtual memory.
pub type ImmutableVirtualRegion<T> = VirtualRegion<false, T>;
/// A region of mutable virtual memory.
pub type MutableVirtualRegion<T> = VirtualRegion<true, T>;

impl<const IS_VIRTUAL: bool, const IS_MUTABLE: bool, T: Alignment>
    Region<IS_VIRTUAL, IS_MUTABLE, T>
{
    const fn new_start_end(pointers: Pointers<IS_VIRTUAL, IS_MUTABLE, T>) -> Self {
        Self(pointers)
    }

    pub fn new(slice: Slice<'_, IS_VIRTUAL, IS_MUTABLE, T>) -> Self {
        let pointers = slice.into_pointers();
        Self::new_start_end(pointers)
    }

    const fn as_pointers(&self) -> &Pointers<IS_VIRTUAL, IS_MUTABLE, T> {
        &self.0
    }

    fn as_immutable_pointers(&self) -> &Pointers<IS_VIRTUAL, false, T> {
        self.0.as_immutable()
    }

    const fn get_starting_pointer(&self) -> &Pointer<IS_VIRTUAL, IS_MUTABLE, T> {
        self.as_pointers().as_smaller()
    }

    const fn get_ending_pointer(&self) -> &Pointer<IS_VIRTUAL, IS_MUTABLE, T> {
        self.as_pointers().as_bigger()
    }

    fn get_length(&self) -> NonZero<usize> {
        self.as_pointers().compute_difference()
    }

    fn intersect_pointer<'b>(
        &'b self,
        pointer: &'b ImmutablePointer<IS_VIRTUAL, T>,
    ) -> bool {
        self.as_immutable_pointers().is_intersecting(pointer)
    }

    fn is_intersecting_pointer(&self, pointer: &Pointer<IS_VIRTUAL, false, T>) -> bool {
        self.intersect_pointer(pointer)
    }

    fn contain_pointer<'b>(
        &'b self,
        pointer: &'b ImmutablePointer<IS_VIRTUAL, T>,
    ) -> bool {
        self.as_immutable_pointers().is_containing(pointer)
    }

    fn is_containing_pointer(&self, pointer: &Pointer<IS_VIRTUAL, false, T>) -> bool {
        self.contain_pointer(pointer)
    }

    pub fn is_intersecting_region(&self, region: &ImmutableRegion<IS_VIRTUAL, T>) -> bool {
        let self_starting_pointer = self.get_starting_pointer();
        let starting_pointer = region.get_starting_pointer();
        let ending_pointer = region.get_ending_pointer();

        self.is_intersecting_pointer(starting_pointer.as_immutable())
            || self.is_containing_pointer(ending_pointer.as_immutable())
            || region.is_intersecting_pointer(self_starting_pointer.as_immutable())
    }
}

impl<const IS_VIRTUAL: bool, const IS_MUTABLE: bool, T: Alignment> core::fmt::Display
    for Region<IS_VIRTUAL, IS_MUTABLE, T>
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "{}", self.as_pointers())
    }
}

/// A region of allocated memory.
#[repr(transparent)]
pub struct AllocatedRegion<'a, const IS_VIRTUAL: bool, T: Alignment> {
    region: MutableRegion<IS_VIRTUAL, T>,
    phantom_data: PhantomData<&'a ()>,
}

impl<const IS_VIRTUAL: bool, T: Alignment> AllocatedRegion<'_, IS_VIRTUAL, T> {
    const fn as_region(&self) -> &MutableRegion<IS_VIRTUAL, T> {
        &self.region
    }

    const fn get_starting_pointer(&self) -> &MutablePointer<IS_VIRTUAL, T> {
        self.as_region().get_starting_pointer()
    }

    const fn get_ending_pointer(&self) -> &MutablePointer<IS_VIRTUAL, T> {
        self.as_region().get_ending_pointer()
    }

    fn get_length(&self) -> NonZero<usize> {
        self.region.get_length()
    }

    pub(crate) fn get_length_bytes(&self) -> NonZero<usize> {
        // SAFETY: the size of an allocated region may never be larger than `isize::MAX` in bytes
        unsafe {
            self.get_length()
                .unchecked_mul(create_non_zero_usize(core::mem::size_of::<T>()))
        }
    }
}

impl<const IS_USER: bool, T: Alignment> core::fmt::Display for AllocatedRegion<'_, IS_USER, T> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "{}", self.as_region())
    }
}

/// A region of allocated physical memory.
pub type PhysicalAllocatedRegion<'a, T> = AllocatedRegion<'a, false, T>;
//pub(crate) type VirtualAllocatedRegion<'a, T: Alignment> = AllocatedRegion<'a, true, T>;

impl<'a, T: Alignment> PhysicalAllocatedRegion<'a, T> {
    pub fn new(slice: MutablePhysicalSlice<'a, T>) -> Self {
        let pointers = slice.into_pointers();
        let region = MutablePhysicalRegion::new_start_end(pointers);

        Self {
            region,
            phantom_data: PhantomData,
        }
    }
}

/// A region of allocated memory associated with memory permissions.
pub struct ProtectedAllocatedRegion<'a, const IS_VIRTUAL: bool, T: Alignment> {
    allocated_region: AllocatedRegion<'a, IS_VIRTUAL, T>,
    protected_length: Cell<NonZero<usize>>,
    permissions: Permissions,
}

/// A region of allocated physical memory associated with memory permissions.
pub type PhysicalProtectedAllocatedRegion<'a, T> = ProtectedAllocatedRegion<'a, false, T>;
/// A region of allocated virtual memory associated with memory permissions.
pub type VirtualProtectedAllocatedRegion<'a, T> = ProtectedAllocatedRegion<'a, true, T>;

impl<'a, const IS_VIRTUAL: bool, T: Alignment> ProtectedAllocatedRegion<'a, IS_VIRTUAL, T> {
    fn new_protect_entirely(
        allocated_region: AllocatedRegion<'a, IS_VIRTUAL, T>,
        permissions: Permissions,
    ) -> Self {
        let protected_length = allocated_region.get_length();

        Self {
            allocated_region,
            protected_length: Cell::new(protected_length),
            permissions,
        }
    }

    pub(crate) fn new(
        allocated_region: AllocatedRegion<'a, IS_VIRTUAL, T>,
        protected_length: NonZero<usize>,
        permissions: Permissions,
    ) -> Result<Self, ()> {
        if protected_length.get() > allocated_region.get_length().get() {
            return Err(());
        }

        let protected_allocated_region = Self {
            allocated_region,
            protected_length: Cell::new(protected_length),
            permissions,
        };

        Ok(protected_allocated_region)
    }

    const fn as_allocated_region(&self) -> &AllocatedRegion<'a, IS_VIRTUAL, T> {
        &self.allocated_region
    }

    pub const fn get_starting_pointer(&self) -> &MutablePointer<IS_VIRTUAL, T> {
        self.as_allocated_region().get_starting_pointer()
    }

    const fn get_allocated_ending_pointer(&self) -> &MutablePointer<IS_VIRTUAL, T> {
        self.as_allocated_region().get_ending_pointer()
    }

    pub fn get_protected_ending_pointer(&self) -> MutablePointer<IS_VIRTUAL, T> {
        let starting_pointer = self.get_starting_pointer();
        let protected_length = self.get_protected_length();
        // SAFETY: a region cannot wrap around
        unsafe { starting_pointer.unchecked_add(protected_length) }
    }

    pub fn get_protected_length(&self) -> NonZero<usize> {
        self.protected_length.get()
    }

    pub fn get_allocated_length(&self) -> NonZero<usize> {
        self.as_allocated_region().get_length()
    }

    pub fn get_protected_length_bytes(&self) -> NonZero<usize> {
        // SAFETY: the size of a protected region may never be larger than `isize::MAX` in bytes
        unsafe {
            self.get_protected_length()
                .unchecked_mul(create_non_zero_usize(core::mem::size_of::<T>()))
        }
    }

    pub const fn get_permissions(&self) -> Permissions {
        self.permissions
    }

    fn resize(&self, length: NonZero<usize>) -> Result<(), ()> {
        if length > self.get_allocated_length() {
            return Err(());
        }

        self.protected_length.set(length);

        Ok(())
    }
}

impl<const IS_USER: bool, T: Alignment> core::fmt::Display
    for ProtectedAllocatedRegion<'_, IS_USER, T>
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "(Start, End): {}; Permissions: {}",
            self.as_allocated_region(),
            self.get_permissions()
        )
    }
}

/// A region of virtual memory mapped to allocated physical memory.
pub struct MappedAllocatedRegion<'a, const IS_USER: bool, T: Alignment> {
    allocated_region: PhysicalAllocatedRegion<'a, T>,
    starting_virtual_pointer: ValidMutableVirtualPointer<IS_USER, T>,
}

//pub(crate) type UserMappedAllocatedRegion<'a, T> = MappedAllocatedRegion<'a, true, T>;
/// A region of kernel virtual memory mapped to allocated physical memory.
pub(crate) type KernelMappedAllocatedRegion<'a, T> = MappedAllocatedRegion<'a, false, T>;

impl<'a, const IS_USER: bool, T: Alignment> MappedAllocatedRegion<'a, IS_USER, T> {
    /// # Safety
    ///
    /// The caller must ensure that the virtual region does not wrap around.
    unsafe fn new_unchecked(
        allocated_region: PhysicalAllocatedRegion<'a, T>,
        starting_virtual_pointer: ValidMutableVirtualPointer<IS_USER, T>,
    ) -> Self {
        Self {
            allocated_region,
            starting_virtual_pointer,
        }
    }

    pub fn new(
        allocated_region: PhysicalAllocatedRegion<'a, T>,
        starting_virtual_pointer: ValidMutableVirtualPointer<IS_USER, T>,
    ) -> Result<Self, ()> {
        let allocated_length = allocated_region.get_length();

        if starting_virtual_pointer
            .checked_add(allocated_length)
            .is_err()
        {
            return Err(());
        }

        let mapped_allocated_region =
            unsafe { Self::new_unchecked(allocated_region, starting_virtual_pointer) };

        Ok(mapped_allocated_region)
    }

    pub fn new_flat(allocated_region: PhysicalAllocatedRegion<'a, T>) -> Self {
        let starting_physical_pointer = allocated_region.get_starting_pointer();
        let starting_virtual_pointer = starting_physical_pointer.to_virtual_pointer();
        // SAFETY: flat mapping is always valid for either userspace or kernel.
        let valid_starting_virtual_pointer =
            unsafe { ValidVirtualPointer::new(starting_virtual_pointer) };

        // SAFETY: a flat-mapped virtual address is identical to a physical region and since no
        // region can wrap around, the flat-mapped virtual address does not wrap around neither.
        let mapped_allocated_region =
            unsafe { Self::new_unchecked(allocated_region, valid_starting_virtual_pointer) };

        mapped_allocated_region
    }

    fn consume(
        self,
    ) -> (
        PhysicalAllocatedRegion<'a, T>,
        ValidMutableVirtualPointer<IS_USER, T>,
    ) {
        (self.allocated_region, self.starting_virtual_pointer)
    }
}

/// A region of virtual memory mapped to allocated physical memory protected by memory permissions.
pub struct MappedProtectedAllocatedRegion<'a, const IS_USER: bool, T: Alignment> {
    physical_protected_allocated_region: PhysicalProtectedAllocatedRegion<'a, T>,
    starting_virtual_pointer: ValidMutableVirtualPointer<IS_USER, T>,
}

/// A region of user virtual memory mapped to allocated physical memory protected by memory
/// permissions.
pub type UserMappedProtectedAllocatedRegion<'a, T> = MappedProtectedAllocatedRegion<'a, true, T>;
/// A region of kernel virtual memory mapped to allocated physical memory protected by memory
/// permissions.
pub(crate) type KernelMappedProtectedAllocatedRegion<'a, T> =
    MappedProtectedAllocatedRegion<'a, false, T>;

impl<'a, const IS_USER: bool, T: Alignment> MappedProtectedAllocatedRegion<'a, IS_USER, T> {
    /// # Safety
    ///
    /// The caller must ensure that the virtual address space covered by the region does not wrap
    /// around.
    unsafe fn new_from_protected_unchecked(
        physical_protected_allocated_region: PhysicalProtectedAllocatedRegion<'a, T>,
        starting_virtual_pointer: ValidMutableVirtualPointer<IS_USER, T>,
    ) -> Self {
        Self {
            physical_protected_allocated_region,
            starting_virtual_pointer,
        }
    }

    pub(crate) fn new_from_protected(
        physical_protected_allocated_region: PhysicalProtectedAllocatedRegion<'a, T>,
        starting_virtual_pointer: ValidMutableVirtualPointer<IS_USER, T>,
    ) -> Result<Self, ()> {
        let allocated_length = physical_protected_allocated_region.get_allocated_length();

        if starting_virtual_pointer
            .checked_add(allocated_length)
            .is_err()
        {
            return Err(());
        }

        // SAFETY: because of the previous if, the virtual address space does not wrap around.
        let mapped_protected_allocated_region = unsafe {
            Self::new_from_protected_unchecked(
                physical_protected_allocated_region,
                starting_virtual_pointer,
            )
        };

        Ok(mapped_protected_allocated_region)
    }

    pub(crate) fn new_from_mapped(
        mapped_allocated_region: MappedAllocatedRegion<'a, IS_USER, T>,
        permissions: Permissions,
    ) -> Self {
        let (allocated_region, starting_virtual_pointer) = mapped_allocated_region.consume();

        let physical_protected_allocated_region =
            PhysicalProtectedAllocatedRegion::new_protect_entirely(allocated_region, permissions);

        // SAFETY: MappedAllocatedRegion guarantees that the virtual address space does not wrap
        // around.
        unsafe {
            Self::new_from_protected_unchecked(
                physical_protected_allocated_region,
                starting_virtual_pointer,
            )
        }
    }

    pub(crate) const fn as_physical_protected_allocated_region(
        &self,
    ) -> &PhysicalProtectedAllocatedRegion<'a, T> {
        &self.physical_protected_allocated_region
    }

    pub const fn get_starting_physical_pointer(&self) -> &MutablePhysicalPointer<T> {
        self.as_physical_protected_allocated_region()
            .get_starting_pointer()
    }

    const fn get_allocated_ending_physical_pointer(&self) -> MutablePhysicalPointer<T> {
        *self
            .as_physical_protected_allocated_region()
            .get_allocated_ending_pointer()
    }

    fn get_protected_ending_physical_pointer(&self) -> MutablePhysicalPointer<T> {
        self.as_physical_protected_allocated_region()
            .get_protected_ending_pointer()
    }

    pub const fn get_starting_virtual_pointer(&self) -> &ValidMutableVirtualPointer<IS_USER, T> {
        &self.starting_virtual_pointer
    }

    pub(crate) fn get_protected_ending_virtual_pointer(
        &self,
    ) -> ValidMutableVirtualPointer<IS_USER, T> {
        let starting_virtual_pointer = self.get_starting_virtual_pointer();
        let protected_length = self.get_protected_length();
        // SAFETY: a region may never wrap
        unsafe { starting_virtual_pointer.unchecked_add(protected_length) }
    }

    pub(crate) fn get_allocated_ending_virtual_pointer(
        &self,
    ) -> ValidMutableVirtualPointer<IS_USER, T> {
        let starting_virtual_pointer = self.get_starting_virtual_pointer();
        let allocated_length = self.get_allocated_length();
        // SAFETY: a region may never wrap
        unsafe { starting_virtual_pointer.unchecked_add(allocated_length) }
    }

    pub fn get_protected_length(&self) -> NonZero<usize> {
        self.as_physical_protected_allocated_region()
            .get_protected_length()
    }

    pub fn get_allocated_length(&self) -> NonZero<usize> {
        self.as_physical_protected_allocated_region()
            .get_allocated_length()
    }

    pub const fn get_permissions(&self) -> Permissions {
        self.as_physical_protected_allocated_region()
            .get_permissions()
    }

    pub(crate) fn is_containing_protected_virtual_byte(
        &self,
        virtual_byte: &ValidImmutableVirtualPointer<IS_USER, u8>,
    ) -> bool {
        let starting_pointer = self.get_starting_virtual_pointer();
        let ending_pointer = self.get_protected_ending_virtual_pointer();

        let starting_address = starting_pointer.get_address();
        let ending_address = ending_pointer.get_address();
        let address = virtual_byte.get_address();

        starting_address <= address && address < ending_address
    }

    fn contain_virtual_pointer<'b>(
        &'b self,
        virtual_pointer: &'b ImmutableVirtualPointer<T>,
    ) -> Result<SmallerPair<&'b ImmutableVirtualPointer<T>>, ()> {
        let starting_pointer = self.get_starting_virtual_pointer().as_virtual_pointer();
        let ending_pointer = self.get_allocated_ending_virtual_pointer();
        let ending_pointer = ending_pointer.as_virtual_pointer();

        if virtual_pointer < ending_pointer.as_immutable() {
            SmallerPair::new(starting_pointer.as_immutable(), virtual_pointer)
        } else {
            Err(())
        }
    }

    fn is_containing_virtual_pointer<'b>(
        &'b self,
        virtual_pointer: &'b ImmutableVirtualPointer<T>,
    ) -> bool {
        self.contain_virtual_pointer(virtual_pointer).is_ok()
    }

    fn intersect_physical_byte<'b, U: AlwaysAligned>(
        &'b self,
        physical_pointer: &'b ImmutablePhysicalPointer<U>,
        ending_physical_pointer: ImmutablePhysicalPointer<U>,
    ) -> Result<SmallerOrEqualPairImmutableReference<'b, ImmutablePhysicalPointer<U>>, ()> {
        let starting_pointer = self.get_starting_physical_pointer();
        let casted_starting_pointer = starting_pointer.infallible_cast_ref();

        if physical_pointer < &ending_physical_pointer {
            SmallerOrEqualPairImmutableReference::new(
                casted_starting_pointer.as_immutable(),
                physical_pointer,
            )
        } else {
            Err(())
        }
    }

    fn intersect_allocated_physical_byte<'b, U: AlwaysAligned>(
        &'b self,
        physical_pointer: &'b ImmutablePhysicalPointer<U>,
    ) -> Result<SmallerOrEqualPairImmutableReference<'b, ImmutablePhysicalPointer<U>>, ()> {
        let ending_pointer = self.get_allocated_ending_physical_pointer();

        self.intersect_physical_byte(
            physical_pointer,
            ending_pointer.infallible_cast().to_immutable(),
        )
    }

    fn intersect_protected_physical_byte<'b, U: AlwaysAligned>(
        &'b self,
        physical_pointer: &'b ImmutablePhysicalPointer<U>,
    ) -> Result<SmallerOrEqualPairImmutableReference<'b, ImmutablePhysicalPointer<U>>, ()> {
        let ending_pointer = self.get_protected_ending_physical_pointer();

        self.intersect_physical_byte(
            physical_pointer,
            ending_pointer.infallible_cast().to_immutable(),
        )
    }

    fn intersect_virtual_byte<'b, U: AlwaysAligned>(
        &'b self,
        virtual_pointer: &'b ValidImmutableVirtualPointer<IS_USER, U>,
        ending_virtual_pointer: ValidImmutableVirtualPointer<IS_USER, U>,
    ) -> Result<
        SmallerOrEqualPairImmutableReference<'b, ValidImmutableVirtualPointer<IS_USER, U>>,
        (),
    > {
        let starting_pointer = self.get_starting_virtual_pointer();
        let casted_starting_pointer = starting_pointer.infallible_cast_ref();

        if virtual_pointer < &ending_virtual_pointer {
            SmallerOrEqualPairImmutableReference::new(
                casted_starting_pointer.as_immutable(),
                virtual_pointer,
            )
        } else {
            Err(())
        }
    }

    fn intersect_protected_virtual_byte<'b, U: AlwaysAligned>(
        &'b self,
        virtual_pointer: &'b ValidImmutableVirtualPointer<IS_USER, U>,
    ) -> Result<
        SmallerOrEqualPairImmutableReference<'b, ValidImmutableVirtualPointer<IS_USER, U>>,
        (),
    > {
        let ending_pointer = self.get_protected_ending_virtual_pointer();

        self.intersect_virtual_byte(
            virtual_pointer,
            ending_pointer.infallible_cast().to_immutable(),
        )
    }

    fn intersect_allocated_virtual_byte<'b, U: AlwaysAligned>(
        &'b self,
        virtual_pointer: &'b ValidImmutableVirtualPointer<IS_USER, U>,
    ) -> Result<
        SmallerOrEqualPairImmutableReference<'b, ValidImmutableVirtualPointer<IS_USER, U>>,
        (),
    > {
        let ending_pointer = self.get_allocated_ending_virtual_pointer();

        self.intersect_virtual_byte(
            virtual_pointer,
            ending_pointer.infallible_cast().to_immutable(),
        )
    }

    fn intersect_virtual_pointer<'b>(
        &'b self,
        virtual_pointer: &'b ImmutableVirtualPointer<T>,
    ) -> Result<SmallerOrEqualPair<&'b ImmutableVirtualPointer<T>>, ()> {
        let starting_pointer = self.get_starting_virtual_pointer();
        let starting_pointer = starting_pointer.as_virtual_pointer();
        let ending_pointer = self.get_allocated_ending_virtual_pointer();
        let ending_pointer = ending_pointer.as_virtual_pointer();
        if virtual_pointer < ending_pointer.as_immutable() {
            SmallerOrEqualPair::new(starting_pointer.as_immutable(), virtual_pointer)
        } else {
            Err(())
        }
    }

    pub(crate) fn is_intersecting_virtual_pointer(
        &self,
        virtual_pointer: &ImmutableVirtualPointer<T>,
    ) -> bool {
        self.intersect_virtual_pointer(virtual_pointer).is_ok()
    }

    pub(super) fn is_intersecting_virtually<const OTHER_IS_USER: bool>(
        &self,
        other: &MappedProtectedAllocatedRegion<OTHER_IS_USER, T>,
    ) -> bool {
        let other_start = other.get_starting_virtual_pointer();
        let other_start = other_start.as_virtual_pointer();
        let other_end = other.get_allocated_ending_virtual_pointer();
        let other_end = other_end.as_virtual_pointer();
        let self_start = self.get_starting_virtual_pointer().as_virtual_pointer();

        self.is_intersecting_virtual_pointer(other_start.as_immutable())
            || self.is_containing_virtual_pointer(other_end.as_immutable())
            || other.is_containing_virtual_pointer(self_start.as_immutable())
    }

    pub(crate) fn resize(&self, length: NonZero<usize>) -> Result<(), ()> {
        self.physical_protected_allocated_region.resize(length)
    }

    fn translate_allocated_physical_pointer_byte<const IS_MUTABLE: bool, U: AlwaysAligned>(
        &self,
        physical_pointer: PhysicalPointer<IS_MUTABLE, U>,
    ) -> Result<ValidVirtualPointer<IS_USER, IS_MUTABLE, U>, PhysicalPointer<IS_MUTABLE, U>> {
        match self.intersect_allocated_physical_byte(physical_pointer.as_immutable()) {
            Err(()) => Err(physical_pointer),
            Ok(smaller_or_equal) => {
                let difference = smaller_or_equal.compute_difference();
                let starting_virtual_pointer = self.get_starting_virtual_pointer();
                let casted_starting_virtual_pointer =
                    starting_virtual_pointer.infallible_cast_ref();
                // SAFETY: a slice may never wrap
                let virtual_pointer =
                    unsafe { casted_starting_virtual_pointer.unchecked_add_zero(difference) };

                Ok(virtual_pointer.cast_mutability())
            }
        }
    }

    fn translate_protected_physical_pointer_byte<const IS_MUTABLE: bool, U: AlwaysAligned>(
        &self,
        physical_pointer: PhysicalPointer<IS_MUTABLE, U>,
    ) -> Result<ValidVirtualPointer<IS_USER, IS_MUTABLE, U>, PhysicalPointer<IS_MUTABLE, U>> {
        match self.intersect_protected_physical_byte(physical_pointer.as_immutable()) {
            Err(()) => Err(physical_pointer),
            Ok(smaller_or_equal) => {
                let difference = smaller_or_equal.compute_difference();
                let starting_virtual_pointer = self.get_starting_virtual_pointer();
                let casted_starting_virtual_pointer =
                    starting_virtual_pointer.infallible_cast_ref();
                // SAFETY: a slice may never wrap
                let virtual_pointer =
                    unsafe { casted_starting_virtual_pointer.unchecked_add_zero(difference) };

                Ok(virtual_pointer.cast_mutability())
            }
        }
    }

    fn translate_protected_virtual_pointer_byte<const IS_MUTABLE: bool, U: AlwaysAligned>(
        &self,
        virtual_pointer: ValidVirtualPointer<IS_USER, IS_MUTABLE, U>,
    ) -> Result<PhysicalPointer<IS_MUTABLE, U>, ValidVirtualPointer<IS_USER, IS_MUTABLE, U>> {
        match self.intersect_protected_virtual_byte(virtual_pointer.as_immutable()) {
            Err(()) => Err(virtual_pointer),
            Ok(smaller_or_equal) => {
                let difference = smaller_or_equal.compute_difference();
                let starting_physical_pointer = self.get_starting_physical_pointer();
                let casted_starting_physical_pointer =
                    starting_physical_pointer.infallible_cast_ref();
                // SAFETY: a slice may never wrap
                let physical_pointer =
                    unsafe { casted_starting_physical_pointer.unchecked_add_zero(difference) };

                Ok(physical_pointer.cast_mutability())
            }
        }
    }

    fn translate_allocated_virtual_pointer_byte<const IS_MUTABLE: bool, U: AlwaysAligned>(
        &self,
        virtual_pointer: ValidVirtualPointer<IS_USER, IS_MUTABLE, U>,
    ) -> Result<PhysicalPointer<IS_MUTABLE, U>, ValidVirtualPointer<IS_USER, IS_MUTABLE, U>> {
        match self.intersect_allocated_virtual_byte(virtual_pointer.as_immutable()) {
            Err(()) => Err(virtual_pointer),
            Ok(smaller_or_equal) => {
                let difference = smaller_or_equal.compute_difference();
                let starting_physical_pointer = self.get_starting_physical_pointer();
                let casted_starting_physical_pointer =
                    starting_physical_pointer.infallible_cast_ref();
                // SAFETY: a slice may never wrap
                let physical_pointer =
                    unsafe { casted_starting_physical_pointer.unchecked_add_zero(difference) };

                Ok(physical_pointer.cast_mutability())
            }
        }
    }
}

impl<const IS_USER: bool, T: Alignment> core::fmt::Display
    for MappedProtectedAllocatedRegion<'_, IS_USER, T>
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "{}; Mapping start: {:#x}",
            self.as_physical_protected_allocated_region(),
            self.get_starting_virtual_pointer(),
        )
    }
}

/// A mapped protected region with a dirty flag.
pub(crate) struct DirtyMappedProtectedAllocatedRegion<'a, const IS_USER: bool, T: Alignment> {
    mapped_protected_allocated_region: MappedProtectedAllocatedRegion<'a, IS_USER, T>,
    is_dirty: Cell<bool>,
}

/// A user mapped protected region with a dirty flag.
pub(crate) type UserDirtyMappedProtectedAllocatedRegion<'a, T> =
    DirtyMappedProtectedAllocatedRegion<'a, true, T>;
/// A kernel mapped protected region with a dirty flag.
pub(crate) type KernelDirtyMappedProtectedAllocatedRegion<'a, T> =
    DirtyMappedProtectedAllocatedRegion<'a, false, T>;

impl<'a, const IS_USER: bool, T: Alignment> DirtyMappedProtectedAllocatedRegion<'a, IS_USER, T> {
    pub(crate) const fn new(
        mapped_protected_allocated_region: MappedProtectedAllocatedRegion<'a, IS_USER, T>,
    ) -> Self {
        Self {
            mapped_protected_allocated_region,
            is_dirty: Cell::new(true),
        }
    }

    pub(crate) const fn as_mapped_protected_allocated_region(
        &self,
    ) -> &MappedProtectedAllocatedRegion<'a, IS_USER, T> {
        &self.mapped_protected_allocated_region
    }

    pub(crate) fn is_dirty(&self) -> bool {
        self.is_dirty.get()
    }

    pub(crate) fn clear_dirty(&self) {
        self.is_dirty.set(false)
    }

    fn set_dirty(&self) {
        self.is_dirty.set(true)
    }

    pub(crate) fn resize(&self, length: NonZero<usize>) -> Result<(), ()> {
        let result = self.as_mapped_protected_allocated_region().resize(length);
        if result.is_ok() {
            self.set_dirty()
        }
        result
    }

    pub(super) fn translate_allocated_physical_pointer_byte<
        const IS_MUTABLE: bool,
        U: AlwaysAligned,
    >(
        &self,
        physical_pointer: PhysicalPointer<IS_MUTABLE, U>,
    ) -> Result<ValidVirtualPointer<IS_USER, IS_MUTABLE, U>, PhysicalPointer<IS_MUTABLE, U>> {
        self.as_mapped_protected_allocated_region()
            .translate_allocated_physical_pointer_byte(physical_pointer)
    }

    pub(super) fn translate_protected_physical_pointer_byte<
        const IS_MUTABLE: bool,
        U: AlwaysAligned,
    >(
        &self,
        physical_pointer: PhysicalPointer<IS_MUTABLE, U>,
    ) -> Result<ValidVirtualPointer<IS_USER, IS_MUTABLE, U>, PhysicalPointer<IS_MUTABLE, U>> {
        self.as_mapped_protected_allocated_region()
            .translate_protected_physical_pointer_byte(physical_pointer)
    }

    pub(super) fn translate_protected_virtual_pointer_byte<
        const IS_MUTABLE: bool,
        U: AlwaysAligned,
    >(
        &self,
        virtual_pointer: ValidVirtualPointer<IS_USER, IS_MUTABLE, U>,
    ) -> Result<PhysicalPointer<IS_MUTABLE, U>, ValidVirtualPointer<IS_USER, IS_MUTABLE, U>> {
        self.as_mapped_protected_allocated_region()
            .translate_protected_virtual_pointer_byte(virtual_pointer)
    }

    pub(super) fn translate_allocated_virtual_pointer_byte<
        const IS_MUTABLE: bool,
        U: AlwaysAligned,
    >(
        &self,
        virtual_pointer: ValidVirtualPointer<IS_USER, IS_MUTABLE, U>,
    ) -> Result<PhysicalPointer<IS_MUTABLE, U>, ValidVirtualPointer<IS_USER, IS_MUTABLE, U>> {
        self.as_mapped_protected_allocated_region()
            .translate_allocated_virtual_pointer_byte(virtual_pointer)
    }
}

impl<const IS_USER: bool, T: Alignment> core::fmt::Display
    for DirtyMappedProtectedAllocatedRegion<'_, IS_USER, T>
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "{}; Dirty: {}",
            self.as_mapped_protected_allocated_region(),
            self.is_dirty(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::memory_management::pages::Page4KiB;
    use crate::memory_management::pointers::{
        VirtualPointer,
        UserVirtualPointer,
        MutableUserVirtualPointer,
    };
    use crate::utilities;
    use crate::utilities::misc::create_non_zero_usize;

    fn create_physical_pointer<T>(address: usize) -> MutablePhysicalPointer<T> {
        // Allocated region
        let pointer = utilities::pointers::MutablePointer::new(address as *mut T).unwrap();
        // SAFETY: let's assume it's a valid physical pointer
        unsafe { MutablePhysicalPointer::new(pointer) }
    }

    fn create_virtual_pointer<T>(address: usize) -> MutableUserVirtualPointer<T> {
        // Allocated region
        let pointer = utilities::pointers::MutablePointer::new(address as *mut T).unwrap();
        // SAFETY: let's assume it's a valid physical pointer
        let virtual_pointer = unsafe { VirtualPointer::new(pointer) };
        // SAFETY: let's assume it's a valid virtual pointer
        unsafe { UserVirtualPointer::new(virtual_pointer) }
    }

    fn create_region(
        starting_physical_address: usize,
        starting_virtual_address: usize,
    ) -> DirtyMappedProtectedAllocatedRegion<'static, true, Page4KiB> {
        let starting_physical_pointer = create_physical_pointer(starting_physical_address);
        let physical_length = create_non_zero_usize(4);
        // SAFETY: let's assume it's a valid physical slice
        let physical_slice = unsafe { MutablePhysicalSlice::from_raw_parts(starting_physical_pointer, physical_length) };
        let allocated_region = AllocatedRegion::new(physical_slice);

        // Protected allocated region
        let permissions = Permissions::ReadWrite;
        let protected_length = create_non_zero_usize(2);
        let protected_allocated_region = ProtectedAllocatedRegion::new(
            allocated_region,
            protected_length,
            permissions,
        ).unwrap();


        // Allocated region
        let starting_virtual_pointer = create_virtual_pointer(starting_virtual_address);
        let mapped_protected_allocated_region = UserMappedProtectedAllocatedRegion::new_from_protected(
            protected_allocated_region,
            starting_virtual_pointer,
        ).unwrap();

        DirtyMappedProtectedAllocatedRegion::new(mapped_protected_allocated_region)
    }

    #[test]
    fn test_resize_dirty_region() {
        let dirty_region = create_region(0x2000_0000, 0x4000_0000);
        assert!(dirty_region.is_dirty());
        dirty_region.clear_dirty();
        assert!(!dirty_region.is_dirty());
        assert!(dirty_region.resize(create_non_zero_usize(3)).is_ok());
        assert!(dirty_region.is_dirty());
        dirty_region.clear_dirty();
        assert!(!dirty_region.is_dirty());
        assert!(dirty_region.resize(create_non_zero_usize(5)).is_err());
        assert!(!dirty_region.is_dirty());
        assert!(dirty_region.resize(create_non_zero_usize(1)).is_ok());
        assert!(dirty_region.is_dirty());
    }

    #[test]
    fn test_translate_allocated_physical_pointer_byte() {
        let dirty_region = create_region(0x2000_0000, 0x4000_0000);

        let physical_pointer = create_physical_pointer::<u8>(0x1FFFFFFF);
        assert!(dirty_region.translate_allocated_physical_pointer_byte(physical_pointer).is_err());

        let physical_pointer = create_physical_pointer::<u8>(0x20000000);
        let virtual_pointer = dirty_region.translate_allocated_physical_pointer_byte(physical_pointer).unwrap();
        assert_eq!(0x40000000, virtual_pointer.get_address().get());

        let physical_pointer = create_physical_pointer::<u8>(0x20003FFF);
        let virtual_pointer = dirty_region.translate_allocated_physical_pointer_byte(physical_pointer).unwrap();
        assert_eq!(0x40003FFF, virtual_pointer.get_address().get());

        let physical_pointer = create_physical_pointer::<u8>(0x20004000);
        assert!(dirty_region.translate_allocated_physical_pointer_byte(physical_pointer).is_err());
    }

    #[test]
    fn test_translate_protected_physical_pointer_byte() {
        let dirty_region = create_region(0x2000_0000, 0x4000_0000);

        let physical_pointer = create_physical_pointer::<u8>(0x1FFFFFFF);
        assert!(dirty_region.translate_protected_physical_pointer_byte(physical_pointer).is_err());

        let physical_pointer = create_physical_pointer::<u8>(0x20000000);
        let virtual_pointer = dirty_region.translate_protected_physical_pointer_byte(physical_pointer).unwrap();
        assert_eq!(0x40000000, virtual_pointer.get_address().get());

        let physical_pointer = create_physical_pointer::<u8>(0x20001FFF);
        let virtual_pointer = dirty_region.translate_protected_physical_pointer_byte(physical_pointer).unwrap();
        assert_eq!(0x40001FFF, virtual_pointer.get_address().get());

        let physical_pointer = create_physical_pointer::<u8>(0x20002000);
        assert!(dirty_region.translate_protected_physical_pointer_byte(physical_pointer).is_err());
    }

    #[test]
    fn test_translate_allocated_virtual_pointer_byte() {
        let dirty_region = create_region(0x2000_0000, 0x4000_0000);

        let virtual_pointer = create_virtual_pointer::<u8>(0x3FFFFFFF);
        assert!(dirty_region.translate_allocated_virtual_pointer_byte(virtual_pointer).is_err());

        let virtual_pointer = create_virtual_pointer::<u8>(0x40000000);
        let virtual_pointer = dirty_region.translate_allocated_virtual_pointer_byte(virtual_pointer).unwrap();
        assert_eq!(0x20000000, virtual_pointer.get_address().get());

        let virtual_pointer = create_virtual_pointer::<u8>(0x40003FFF);
        let virtual_pointer = dirty_region.translate_allocated_virtual_pointer_byte(virtual_pointer).unwrap();
        assert_eq!(0x20003FFF, virtual_pointer.get_address().get());

        let virtual_pointer = create_virtual_pointer::<u8>(0x40004000);
        assert!(dirty_region.translate_allocated_virtual_pointer_byte(virtual_pointer).is_err());
    }

    #[test]
    fn test_translate_protected_virtual_pointer_byte() {
        let dirty_region = create_region(0x2000_0000, 0x4000_0000);

        let virtual_pointer = create_virtual_pointer::<u8>(0x3FFFFFFF);
        assert!(dirty_region.translate_protected_virtual_pointer_byte(virtual_pointer).is_err());

        let virtual_pointer = create_virtual_pointer::<u8>(0x40000000);
        let virtual_pointer = dirty_region.translate_protected_virtual_pointer_byte(virtual_pointer).unwrap();
        assert_eq!(0x20000000, virtual_pointer.get_address().get());

        let virtual_pointer = create_virtual_pointer::<u8>(0x40001FFF);
        let virtual_pointer = dirty_region.translate_protected_virtual_pointer_byte(virtual_pointer).unwrap();
        assert_eq!(0x20001FFF, virtual_pointer.get_address().get());

        let virtual_pointer = create_virtual_pointer::<u8>(0x40002000);
        assert!(dirty_region.translate_protected_virtual_pointer_byte(virtual_pointer).is_err());
    }

    #[test]
    fn test_is_intersecting_virtually() {
        let dirty_region1 = create_region(0x2000_0000, 0x4000_0000);
        let dirty_region2 = create_region(0x2000_4000, 0x4000_2000);

        let region1 = dirty_region1.as_mapped_protected_allocated_region();
        let region2 = dirty_region2.as_mapped_protected_allocated_region();
        assert!(region1.is_intersecting_virtually(region2));

        let dirty_region1 = create_region(0x2000_0000, 0x4000_0000);
        let dirty_region2 = create_region(0x2000_4000, 0x4000_4000);

        let region1 = dirty_region1.as_mapped_protected_allocated_region();
        let region2 = dirty_region2.as_mapped_protected_allocated_region();
        assert!(!region1.is_intersecting_virtually(region2));
    }
}
