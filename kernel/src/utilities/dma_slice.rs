// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Leon Schuermann <leon@is.currently.online> 2026.
// Copyright Tock Contributors 2026.

//! Mechanism for sharing buffers with DMA peripherals.

use core::marker::PhantomData;
use core::ops::Range;
use core::ptr::{self, NonNull};

use super::leasable_buffer::{SubSlice, SubSliceMut, SubSliceMutImmut};
use crate::platform::dma_fence::DmaFence;

/// An immutable buffer that can be safely used for read-only DMA operations.
///
/// Creating a [`DmaSlice`] over an immutable Rust slice ensures that all prior
/// Rust writes to this slice are observable by any DMA operations initiated
/// through an MMIO write operation, where that MMIO write is performed after
/// constructing the [`DmaSlice`].
///
/// For this guarantee to hold, the [`DmaSlice`] struct must exist for the
/// duration of the entire DMA operation, until the Rust program has observed
/// that the operation is complete (such as by reading a status bit in memory or
/// an MMIO register).
///
/// [`DmaSlice`] wraps an immutable, shared Rust slice reference. As such, its
/// contents must not be modified by the DMA operation. For a DMA operation that
/// may write to the supplied buffer, use [`DmaSliceMut`] instead.
#[derive(Debug)]
pub struct DmaSlice<'a, T> {
    slice: &'a [T],
}

impl<'a, T> DmaSlice<'a, T> {
    /// Create a [`DmaSlice`] from a shared, immutable Rust slice.
    ///
    /// This function uses the supplied `fence` object to ensure that all prior
    /// writes to `slice` are exposed to any DMA operations initiated by an MMIO
    /// read or write operation after this function returns, and which finish
    /// before the resulting [`DmaSlice`] is dropped.
    pub fn from_slice_ref(slice: &[T], fence: impl DmaFence) -> DmaSlice<'_, T> {
        // Ensure that all prior writes to this slice are exposed to any DMA
        // operations initiated by an MMIO read or write operation after this
        // function returns.
        fence.release::<T>(ptr::from_ref(slice) as *mut [T]);

        DmaSlice { slice }
    }

    /// Returns the pointer to the first element of the wrapped slice reference.
    pub fn as_ptr(&self) -> *const T {
        self.slice.as_ptr()
    }

    /// Returns the length of the wrapped slice reference.
    pub fn len(&self) -> usize {
        self.slice.len()
    }

    /// Retrieve the inner slice reference.
    pub fn as_slice_ref(&self) -> &'a [T] {
        self.slice
    }
}

/// A mutable buffer that can be safely used for DMA operations that read from
/// and/or write to the buffer's contents.
///
/// Creating a [`DmaSliceMut`] over a mutable Rust slice ensures that all prior
/// Rust writes to this slice are observable by any DMA operations initiated
/// through an MMIO write operation, where that MMIO write is performed
/// **after** constructing the `DmaSliceMut`. All writes by the DMA operation
/// will be observable by Rust when calling
/// [`restore_mut_slice_ref`](Self::restore_mut_slice_ref) **after** the DMA
/// operation is finished.
///
/// # Safety Considerations
///
/// Users **must** eventually call
/// [`restore_mut_slice_ref`](Self::restore_mut_slice_ref) to retrieve the
/// underlying buffer. The [`DmaSliceMut`] must exist for the entire duration of
/// the DMA operation. Users must never drop a [`DmaSliceMut`] with a
/// non-`'static` lifetime, as this could provide access to the underlying
/// buffer without guaranteeing that the DMA operation has finished, and without
/// issuing a DMA memory fence to ensure that writes by the DMA operation are
/// visible to Rust.
///
/// [`restore_mut_slice_ref`](Self::restore_mut_slice_ref) must only be called
/// after the DMA operation has been observed to be complete (such as through a
/// memory or MMIO read). Callers must ensure that the hardware will not perform
/// any further writes to the buffer at the point where
/// [`restore_mut_slice_ref`](Self::restore_mut_slice_ref) is called.
///
/// Callers must further ensure that they start DMA operations only after
/// constructing the [`DmaSliceMut`], and only in the memory region described by
/// [`as_mut_ptr`](Self::as_mut_ptr) and [`len`](Self::len).
///
/// Users are responsible to ensure that, after the DMA operation completes and
/// before calling [`restore_mut_slice_ref`](Self::restore_mut_slice_ref), every
/// element in the underlying slice represents a well-initialized and valid
/// instance of its type (with the exception of padding bytes). See the
/// [zerocopy crate](https://docs.rs/zerocopy/0.8.31/zerocopy/) for an more
/// in-depth explanation of these requirements.
#[derive(Debug)]
pub struct DmaSliceMut<'a, T> {
    slice_ptr: NonNull<[T]>,
    _lt: PhantomData<&'a mut [T]>,
}

impl<'a, T> DmaSliceMut<'a, T> {
    /// Create a [`DmaSliceMut`] from a unique, mutable Rust slice.
    ///
    /// This function uses the supplied `fence` object to ensure that all prior
    /// writes to `slice` are exposed to any DMA operations initiated by an MMIO
    /// read or write operation after this function returns, and which finish
    /// before calling [`restore_mut_slice_ref`](Self::restore_mut_slice_ref).
    ///
    /// # Safety
    ///
    /// Refer the safety documentation of the [`DmaSliceMut`] type.
    ///
    /// This function is unsafe, as dropping or
    /// [`forget`](core::mem::forget)ting its return value is not allowed when
    /// the lifetime `'b` is not `'static`. Users **must** eventually call
    /// [`restore_mut_slice_ref`](Self::restore_mut_slice_ref) to retrieve the
    /// underlying buffer.
    #[must_use]
    pub unsafe fn from_mut_slice_ref(slice: &mut [T], fence: impl DmaFence) -> DmaSliceMut<'_, T> {
        let dma_slice_mut = DmaSliceMut {
            slice_ptr: NonNull::from_mut(slice),
            _lt: PhantomData,
        };

        // Ensure that all prior writes to this slice are exposed to any DMA
        // operations initiated by an MMIO read or write operation after this
        // function returns:
        fence.release::<T>(dma_slice_mut.slice_ptr.as_ptr());

        dma_slice_mut
    }

    /// Create a [`DmaSliceMut`] from a unique, mutable Rust slice with
    /// `'static` lifetime.
    ///
    /// This function uses the supplied `fence` object to ensure that all prior
    /// writes to `slice` are exposed to any DMA operations initiated by an MMIO
    /// read or write operation after this function returns, and which finish
    /// before calling [`restore_mut_slice_ref`](Self::restore_mut_slice_ref).
    ///
    /// # Comparsion with [`from_mut_slice_ref`](Self::from_mut_slice_ref)
    ///
    /// In contrast to [`from_mut_slice_ref`](Self::from_mut_slice_ref) this
    /// function is safe, as dropping or forgetting its return value is safe, it
    /// would merely leak memory and make the underlying slice inaccessible.
    ///
    /// The other safety considerations of the [`DmaSliceMut`] type still apply.
    pub fn from_static_mut_slice_ref(
        slice: &'static mut [T],
        fence: impl DmaFence,
    ) -> DmaSliceMut<'static, T> {
        unsafe { Self::from_mut_slice_ref(slice, fence) }
    }

    /// Returns the pointer to the first element of the wrapped slice reference.
    pub fn as_mut_ptr(&self) -> *mut T {
        // TODO: switch `.cast()` to `.as_mut_ptr()` on the slice pointer (`*mut
        // [T]`) to obtain the "thin", raw pointer to its first element. This is
        // blocked on the nightly `slice_ptr_get` feature.
        self.slice_ptr.as_ptr().cast()
    }

    /// Returns the length of the wrapped slice reference.
    pub fn len(&self) -> usize {
        self.slice_ptr.len()
    }

    /// Recover the original, unique (mutable) slice used to construct this
    /// [`DmaSliceMut`] object.
    ///
    /// This function uses the supplied `fence` object to ensure that all prior
    /// writes to `slice` by any completed DMA operations are exposed to any
    /// subsequent Rust reads.
    ///
    /// # Safety
    ///
    /// Refer the safety documentation of the [`DmaSliceMut`] type.
    ///
    /// In particular, [`restore_mut_slice_ref`](Self::restore_mut_slice_ref)
    /// must only be called after the DMA operation has been observed to be
    /// complete (such as through a memory or MMIO read). Callers must ensure
    /// that the hardware will not perform any further writes to the buffer at
    /// the point where [`restore_mut_slice_ref`](Self::restore_mut_slice_ref)
    /// is called.
    pub unsafe fn restore_mut_slice_ref(mut self, fence: impl DmaFence) -> &'a mut [T] {
        // Ensure that any reads from Rust to the buffer described by
        // `slice_ptr` _after_ this function returns reflect all writes made by
        // DMA operations finished _before_ this function ran:
        fence.acquire::<T>(self.slice_ptr.as_ptr());

        unsafe { self.slice_ptr.as_mut() }
    }
}

/// A buffer that can be safely used for read-only DMA operations, backed by
/// either a shared (immutable) or unique (mutable) Rust slice.
///
/// Creating a [`DmaSliceMutImmut`] over a Rust slice ensures that all prior
/// Rust writes to this slice are observable by any DMA operations initiated
/// through an MMIO write operation, where that MMIO write is performed after
/// constructing the `DmaSliceMutImmut`.
///
/// For this guarantee to hold, the `DmaSliceMutImmut` instance must exist for
/// the duration of the entire DMA operation, until the Rust program has
/// observed that the operation is complete (such as by reading a status bit in
/// memory or an MMIO register).
///
/// [`DmaSliceMutImmut`] may wrap an immutable, shared Rust slice
/// reference. Furthermore, in contrast to `DmaSliceMut`, `DmaSliceMutImmut`
/// may not expose writes performed by a DMA operation back to Rust. As such,
/// its contents *must* not be modified by the DMA operation. For a DMA
/// operation that may write to the supplied buffer, use [`DmaSliceMut`]
/// instead.
#[derive(Debug)]
pub enum DmaSliceMutImmut<'a, T> {
    Immutable(DmaSlice<'a, T>),
    Mutable(DmaSliceMut<'a, T>),
}

impl<'a, T> DmaSliceMutImmut<'a, T> {
    /// Create a [`DmaSliceMutImmut`] from a shared, immutable Rust slice.
    ///
    /// This function uses the supplied `fence` object to ensure that all prior
    /// writes to `slice` are exposed to any DMA operations initiated by an MMIO
    /// read or write operation after this function returns, and which finish
    /// before the resulting [`DmaSliceMutImmut`] is dropped.
    pub fn from_slice_ref(slice: &[T], fence: impl DmaFence) -> DmaSliceMutImmut<'_, T> {
        DmaSliceMutImmut::Immutable(DmaSlice::from_slice_ref(slice, fence))
    }

    /// Create a [`DmaSliceMutImmut`] from a unique, mutable Rust slice.
    ///
    /// This function uses the supplied `fence` object to ensure that all prior
    /// writes to `slice` are exposed to any DMA operations initiated by an MMIO
    /// read or write operation after this function returns, and which finish
    /// before the resulting [`DmaSliceMutImmut`] is dropped.
    ///
    /// Even though this method takes a unique, mutable Rust slice, DMA
    /// operations must not modify the buffers contents.
    pub fn from_mut_slice_ref(slice: &mut [T], fence: impl DmaFence) -> DmaSliceMutImmut<'_, T> {
        // # Safety
        //
        // `DmaSliceMut::from_mut_slice_ref` is unsafe, as dropping its return
        // value without calling `restore_mut_slice_ref` may make the underlying
        // buffer accessible as a Rust slice, potentially before the DMA
        // operation is complete, and without using `fence.acquire` to make DMA
        // writes visible to Rust. However, this struct does not permit DMA
        // operations which write to the slice, and hence it can be safely
        // dropped without risk of concurrent modifications or incoherence.
        DmaSliceMutImmut::Mutable(unsafe { DmaSliceMut::from_mut_slice_ref(slice, fence) })
    }

    /// Returns the pointer to the first element of the wrapped slice reference.
    pub fn as_ptr(&self) -> *const T {
        match self {
            DmaSliceMutImmut::Immutable(dma_slice) => dma_slice.as_ptr(),
            DmaSliceMutImmut::Mutable(dma_slice_mut) => dma_slice_mut.as_mut_ptr() as *const T,
        }
    }

    /// Returns the length of the wrapped slice reference.
    pub fn len(&self) -> usize {
        match self {
            DmaSliceMutImmut::Immutable(dma_slice) => dma_slice.len(),
            DmaSliceMutImmut::Mutable(dma_slice_mut) => dma_slice_mut.len(),
        }
    }

    /// Retrieve the inner slice reference.
    pub fn as_slice_ref(&self) -> &'a [T] {
        match self {
            DmaSliceMutImmut::Immutable(dma_slice) => dma_slice.as_slice_ref(),
            DmaSliceMutImmut::Mutable(dma_slice_mut) => unsafe {
                // # Safety
                //
                // Over the duration that [`DmaSliceMutImmut`] the user
                // guarantees that no DMA operation modifies the buffer (and
                // doing so would require an MMIO write, which is itself
                // unsafe). The `dma_slice_mut` is capturing a unique, mutable
                // borrow of the underlying slice over its lifetime `'a`. As
                // such, we can safely hand out immutable references over this
                // slice, which are also bound to the lifetime `'a`.
                core::slice::from_raw_parts(
                    dma_slice_mut.as_mut_ptr() as *const T,
                    dma_slice_mut.len(),
                )
            },
        }
    }
}

/// An immutable buffer that can be safely used for read-only DMA operations,
/// created from a `SubSlice` describing an active range in a larger buffer.
///
/// Creating a [`DmaSubSlice`] over a [`SubSlice`] ensures that all prior Rust
/// writes to the active region of this slice are observable by any DMA
/// operations initiated through an MMIO write operation, where that MMIO write
/// is performed *after* constructing the `DmaSubSlice`.
///
/// For this guarantee to hold, the `DmaSubSlice` struct must exist for the
/// duration of the entire DMA operation, until the Rust program has observed
/// that the operation is complete (such as by reading a status bit in memory or
/// an MMIO register).
///
/// [`DmaSubSlice`] wraps an immutable, shared Rust slice reference. As such,
/// its contents must not be modified by the DMA operation. For a DMA operation
/// that may write to the supplied buffer, use [`DmaSubSliceMut`] instead.
#[derive(Debug)]
pub struct DmaSubSlice<'a, T> {
    sub_slice: SubSlice<'a, T>,
}

impl<'a, T> DmaSubSlice<'a, T> {
    /// Create a [`DmaSubSlice`] from a shared, immutable Rust slice.
    ///
    /// This function uses the supplied `fence` object to ensure that all prior
    /// writes to `slice` are exposed to any DMA operations initiated by an MMIO
    /// read or write operation after this function returns, and which finish
    /// before the resulting [`DmaSubSlice`] is dropped.
    pub fn from_sub_slice(sub_slice: SubSlice<'_, T>, fence: impl DmaFence) -> DmaSubSlice<'_, T> {
        // Ensure that all prior writes to this slice are exposed to any DMA
        // operations initiated by an MMIO read or write operation after this
        // function returns:
        //
        // Clippy says we should be using `.as_mut_ptr()` instead of `.as_ptr()
        // as *mut T`, but that method doesn't exist. The cast doesn't matter
        // here, `DmaFence::release` will not actually dereference the memory.
        #[allow(clippy::as_ptr_cast_mut)]
        fence.release::<T>(ptr::slice_from_raw_parts_mut(
            // `SubSlice::as_ptr()` returns a pointer to the currently
            // accessible portion of the `SubSlice`.
            sub_slice.as_ptr() as *mut T,
            // `SubSlice::len()` returns the length of the currently accessible
            // portion of the `SubSlice`.
            sub_slice.len(),
        ));

        DmaSubSlice { sub_slice }
    }

    /// Returns the pointer to the first element of the currently accessible
    /// portion of the wrapped `SubSlice`.
    pub fn as_ptr(&self) -> *const T {
        self.sub_slice.as_ptr()
    }

    /// Returns the length of the currently accessible range of the wrapped
    /// `SubSlice`.
    pub fn len(&self) -> usize {
        self.sub_slice.len()
    }

    /// Retrieve the wrapped `SubSlice`.
    pub fn as_sub_slice(&self) -> SubSlice<'a, T> {
        self.sub_slice
    }
}

/// An mutable buffer that can be safely used for DMA operations that read from
/// and/or write to the buffer's contents.
///
/// Creating a [`DmaSubSliceMut`] over a [`SubSliceMut`] ensures that all prior
/// Rust writes to this slice's active range are observable by any DMA
/// operations initiated through an MMIO write operation, where that MMIO write
/// is performed **after** constructing the `DmaSubSliceMut`. All writes by the
/// DMA operation will be observable by Rust when calling
/// [`restore_sub_slice_mut`](Self::restore_sub_slice_mut) **after** the DMA
/// operation is finished.
///
/// # Safety
///
/// Users **must** eventually call
/// [`restore_sub_slice_mut`](Self::restore_sub_slice_mut) to retrieve the
/// underlying buffer. The [`DmaSubSliceMut`] must exist for the entire duration
/// of the DMA operation. Users must never drop a [`DmaSubSliceMut`] with a
/// non-`'static` lifetime, as this could provide access to the underlying
/// buffer without guaranteeing that the DMA operation has finished, and without
/// issuing a DMA memory fence to ensure that writes by the DMA operation are
/// visible to Rust.
///
/// [`restore_sub_slice_mut`](Self::restore_sub_slice_mut) must only be called
/// after the DMA operation has been observed to be complete (such as through a
/// memory or MMIO read). Callers must ensure that the hardware will not perform
/// any further writes to the buffer at the point where
/// [`restore_sub_slice_mut`](Self::restore_sub_slice_mut) is called.
///
/// Callers must further ensure that they start DMA operations only after
/// constructing the [`DmaSubSliceMut`], and only in the memory region described
/// by [`as_mut_ptr`](Self::as_mut_ptr) and [`len`](Self::len).
///
/// Users are responsible to ensure that, after the DMA operation completes and
/// before calling [`restore_mut_slice_ref`](Self::restore_sub_slice_mut), every
/// element in the underlying slice represents a well-initialized and valid
/// instance of its type (with the exception of padding bytes). See the
/// [zerocopy crate](https://docs.rs/zerocopy/0.8.31/zerocopy/) for an more
/// in-depth explanation of these requirements.
#[derive(Debug)]
pub struct DmaSubSliceMut<'a, T> {
    internal_slice_ptr: NonNull<[T]>,
    active_range: Range<usize>,
    _lt: PhantomData<&'a mut [T]>,
}

impl<'a, T> DmaSubSliceMut<'a, T> {
    /// Create a [`DmaSubSliceMut`] from a [`SubSliceMut`].
    ///
    /// This function uses the supplied `fence` object to ensure that all prior
    /// writes to the active region of `slice` are exposed to any DMA operations
    /// initiated by an MMIO read or write operation after this function
    /// returns, and which finish before calling
    /// [`restore_sub_slice_mut`](Self::restore_sub_slice_mut).
    ///
    /// # Safety
    ///
    /// Refer the safety documentation of the [`DmaSubSliceMut`] type.
    ///
    /// This function is unsafe, as dropping or
    /// [`forget`](core::mem::forget)ting its return value is not allowed when
    /// the lifetime `'b` is not `'static`. Users **must** eventually call
    /// [`restore_sub_slice_mut`](Self::restore_sub_slice_mut) to retrieve the
    /// underlying buffer.
    #[must_use]
    pub unsafe fn from_sub_slice_mut(
        sub_slice_mut: SubSliceMut<'_, T>,
        fence: impl DmaFence,
    ) -> DmaSubSliceMut<'_, T> {
        let active_range = sub_slice_mut.active_range();
        let internal_slice_ptr = sub_slice_mut.take();

        // Store only a fat raw pointer to the inner slice:
        let dma_sub_slice_mut = DmaSubSliceMut {
            internal_slice_ptr: NonNull::from_mut(internal_slice_ptr),
            active_range,
            _lt: PhantomData,
        };

        // Ensure that all prior writes to the currently active portion of this
        // SubSliceMut are exposed to any DMA operations initiated by an MMIO
        // read or write operation after this function returns:
        fence.release::<T>(ptr::slice_from_raw_parts_mut(
            dma_sub_slice_mut.as_mut_ptr(),
            dma_sub_slice_mut.len(),
        ));

        dma_sub_slice_mut
    }

    /// Create a [`DmaSubSliceMut`] from a [`SubSliceMut`] with `'static`
    /// lifetime.
    ///
    /// This function uses the supplied `fence` object to ensure that all prior
    /// writes to `slice` are exposed to any DMA operations initiated by an MMIO
    /// read or write operation after this function returns, and which finish
    /// before calling [`restore_sub_slice_mut`](Self::restore_sub_slice_mut).
    ///
    /// # Safety
    ///
    /// Refer the safety documentation of the [`DmaSubSliceMut`] type.
    ///
    /// In contrast to `from_slice_ref` this function is safe, as dropping or
    /// forgetting its return value is safe, it would merely leak memory and
    /// make the underlying slice inaccessible.
    pub fn from_static_sub_slice_mut(
        sub_slice: SubSliceMut<'static, T>,
        fence: impl DmaFence,
    ) -> DmaSubSliceMut<'static, T> {
        unsafe { Self::from_sub_slice_mut(sub_slice, fence) }
    }

    /// Returns the pointer to the first element of the active range of the
    /// wrapped [`SubSliceMut`].
    pub fn as_mut_ptr(&self) -> *mut T {
        // TODO: switch `.cast()` to `.as_mut_ptr()` on the slice pointer (`*mut
        // [T]`) to obtain the "thin", raw pointer to its first element. This is
        // blocked on the nightly `slice_ptr_get` feature.
        self.internal_slice_ptr.as_ptr().cast::<T>().wrapping_add(
            if self.active_range.start >= self.internal_slice_ptr.len() {
                // `range.start` is out of bounds, return a pointer that's one
                // after the last byte in this buffer, and length `0`:
                self.internal_slice_ptr.len()
            } else {
                // Start is in bounds:
                self.active_range.start
            },
        )
    }

    /// Returns the length of the active range of the wrapped [`SubSliceMut`].
    pub fn len(&self) -> usize {
        core::cmp::min(
            self.active_range
                .end
                .saturating_sub(self.active_range.start),
            self.internal_slice_ptr.len(),
        )
    }

    /// Recover the original [`SubSliceMut`] used to construct this
    /// [`DmaSubSliceMut`] object.
    ///
    /// This function uses the supplied `fence` object to ensure that all prior
    /// writes to the active range of `slice` by any completed DMA operations
    /// are exposed to any subsequent Rust reads.
    ///
    /// # Safety
    ///
    /// Refer the safety documentation of the [`DmaSubSliceMut`] type.
    ///
    /// In particular, [`restore_sub_slice_mut`](Self::restore_sub_slice_mut)
    /// must only be called after the DMA operation has been observed to be
    /// complete (such as through a memory or MMIO read). Callers must ensure
    /// that the hardware will not perform any further writes to the buffer at
    /// the point where [`restore_sub_slice_mut`](Self::restore_sub_slice_mut)
    /// is called.
    pub unsafe fn restore_sub_slice_mut(mut self, fence: impl DmaFence) -> SubSliceMut<'a, T> {
        // Ensure that any reads from Rust to the active range of the buffer
        // (described by `self.as_mut_ptr()` and `self.len()`) _after_ this
        // function returns reflect all writes made by DMA operations finished
        // _before_ this function ran:
        fence.acquire::<T>(ptr::slice_from_raw_parts_mut(self.as_mut_ptr(), self.len()));

        // Restore the original `SubSliceMut` configuration:
        let mut sub_slice_mut = SubSliceMut::new(unsafe { self.internal_slice_ptr.as_mut() });
        sub_slice_mut.slice(self.active_range);
        sub_slice_mut
    }

    /// Recover the original [`SubSliceMut`] used to construct this
    /// [`DmaSubSliceMut`] object, without performing an acquire DMA fence.
    ///
    /// # Safety
    ///
    /// Refer the safety documentation of the [`DmaSubSliceMut`] type.
    ///
    /// This function does not necessarily expose any writes to the underlying
    /// buffer to Rust. It must only be used when the underlying buffer's
    /// contents have not been modified by a DMA operation (i.e., any DMA
    /// operations operating on the buffer while this `DmaSubSliceMut` existed
    /// were read-only).
    unsafe fn restore_sub_slice_mut_no_acquire(mut self) -> SubSliceMut<'a, T> {
        // Restore the original `SubSliceMut` configuration:
        let mut sub_slice_mut = SubSliceMut::new(unsafe { self.internal_slice_ptr.as_mut() });
        sub_slice_mut.slice(self.active_range);
        sub_slice_mut
    }
}

/// A buffer that can be safely used for read-only DMA operations, backed by
/// either a [`SubSliceMutImmut`].
///
/// Creating a [`DmaSubSliceMutImmut`] over a [`SubSliceMutImmut`] ensures that all
/// prior Rust writes to the active region of this slice are observable by any
/// DMA operations initiated through an MMIO write operation, where that MMIO
/// write is performed *after* constructing the `DmaSubSliceMutImmut`.
///
/// For this guarantee to hold, the `DmaSubSliceMutImmut` struct must exist for the
/// duration of the entire DMA operation, until the Rust program has observed
/// that the operation is complete (such as by reading a status bit in memory or
/// an MMIO register).
///
/// [`DmaSliceMutImmut`] may wrap an immutable, shared Rust slice
/// reference. Furthermore, in contrast to `DmaSubSliceMut`,
/// `DmaSubSliceMutImmut` may not expose writes performed by a DMA operation
/// back to Rust. As such, its contents *must* not be modified by the DMA
/// operation. For a DMA operation that may write to the active range of the
/// supplied sub slice, use [`DmaSubSliceMut`] instead.
#[derive(Debug)]
pub enum DmaSubSliceMutImmut<'a, T> {
    Immutable(DmaSubSlice<'a, T>),
    Mutable(DmaSubSliceMut<'a, T>),
}

impl<'a, T> DmaSubSliceMutImmut<'a, T> {
    /// Create a [`DmaSubSliceMutImmut`] from a [`SubSliceMutImmut`].
    ///
    /// This function uses the supplied `fence` object to ensure that all prior
    /// writes to `slice` are exposed to any DMA operations initiated by an MMIO
    /// read or write operation after this function returns, and which finish
    /// before the resulting [`DmaSubSlice`] is dropped.
    pub fn from_sub_slice_mut_immut(
        sub_slice: SubSliceMutImmut<'_, T>,
        fence: impl DmaFence,
    ) -> DmaSubSliceMutImmut<'_, T> {
        match sub_slice {
            SubSliceMutImmut::Immutable(sub_slice) => {
                DmaSubSliceMutImmut::Immutable(DmaSubSlice::from_sub_slice(sub_slice, fence))
            }
            SubSliceMutImmut::Mutable(sub_slice_mut) => DmaSubSliceMutImmut::Mutable(unsafe {
                // # Safety
                //
                // `DmaSubSliceMut::from_sub_slice_mut` is unsafe, as dropping
                // its return value without calling `restore_sub_slice_mut` may
                // make the underlying buffer accessible as a Rust slice,
                // potentially before the DMA operation is complete, and without
                // using `fence.acquire` to make DMA writes visible to
                // Rust. However, this struct does not permit DMA operations
                // which write to the slice, and hence it can be safely dropped
                // without risk of concurrent modifications or incoherence.
                DmaSubSliceMut::from_sub_slice_mut(sub_slice_mut, fence)
            }),
        }
    }

    /// Returns the pointer to the first element of the currently accessible
    /// portion of the wrapped `SubSlice`.
    pub fn as_ptr(&self) -> *const T {
        match self {
            DmaSubSliceMutImmut::Immutable(dma_sub_slice) => dma_sub_slice.as_ptr(),
            DmaSubSliceMutImmut::Mutable(dma_sub_slice_mut) => {
                dma_sub_slice_mut.as_mut_ptr() as *const T
            }
        }
    }

    /// Returns the length of the currently accessible range of the wrapped
    /// `SubSlice`.
    pub fn len(&self) -> usize {
        match self {
            DmaSubSliceMutImmut::Immutable(dma_sub_slice) => dma_sub_slice.len(),
            DmaSubSliceMutImmut::Mutable(dma_sub_slice_mut) => dma_sub_slice_mut.len(),
        }
    }

    /// Reconstruct the original `SubSliceMutImmut`.
    ///
    /// This function must be called only when the Rust program has observed
    /// that the DMA operation over the buffer is complete (such as by reading a
    /// status bit in memory or an MMIO register). Otherwise, any future reads
    /// by the DMA peripheral may not be consistent with the buffer contents.
    pub fn restore_sub_slice_mut_immut(self) -> SubSliceMutImmut<'a, T> {
        // We don't need to perform an `acquire` fence, as any DMA operation on
        // the underlying slice must not have changed its contents:
        match self {
            DmaSubSliceMutImmut::Immutable(dma_sub_slice) => {
                SubSliceMutImmut::Immutable(dma_sub_slice.as_sub_slice())
            }
            DmaSubSliceMutImmut::Mutable(dma_sub_slice_mut) => SubSliceMutImmut::Mutable(unsafe {
                // # Safety
                //
                // The user guarantees that there has not been any DMA operation
                // that changed the buffers contents while the
                // `DmaSubSliceMutImmut` existed, and hence restoring a unique
                // Rust slice through `restore_sub_slice_mut` is safe. No
                // acquire-fence is needed, given the bufer contents have not
                // been modified.
                dma_sub_slice_mut.restore_sub_slice_mut_no_acquire()
            }),
        }
    }
}

#[cfg(test)]
mod miri_tests {
    use core::ptr;

    use super::super::leasable_buffer::{SubSlice, SubSliceMut};
    use super::{DmaSlice, DmaSliceMut, DmaSubSlice, DmaSubSliceMut};

    /// A mock fence that does nothing, as Miri operations are sequential within
    /// a single thread for this test.
    #[derive(Debug, Clone, Copy)]
    struct MockFence {
        _private: (),
    }

    impl MockFence {
        /// `MockFence` does not actually deliver any guarantees and is only
        /// used for testing, thus make it unsafe to construct:
        unsafe fn new() -> Self {
            MockFence { _private: () }
        }
    }

    unsafe impl super::DmaFence for MockFence {
        fn release<T>(self, _buf: *mut [T]) {
            // In a real system, this flushes caches. In Miri, memory is
            // perfectly coherent, and our "DMA" reads are simply raw pointer
            // reads.
        }

        fn acquire<T>(self, _buf: *mut [T]) {
            // In a real system, this invalidates caches. In Miri, memory is
            // perfectly coherent, and our "DMA" writes are simply raw pointer
            // writes.
        }
    }

    // Helper to simulate a DMA peripheral writing to memory. This writes to the
    // memory using the pointer exposed by the DMA wrapper, which is legal
    // because the wrapper owns the mutable borrow.
    unsafe fn simulate_dma_write<T: Copy>(dst: *mut T, val: T, offset: usize) {
        let target = dst.add(offset);
        ptr::write(target, val);
    }

    #[test]
    fn test_dma_slice_immut_basic() {
        let fence = unsafe { MockFence::new() };

        let data = [10u8, 20, 30, 40];

        // 1. Create DmaSlice
        let dma = DmaSlice::from_slice_ref(&data, fence);

        // 2. Verify properties
        assert_eq!(dma.len(), 4);
        assert_eq!(dma.as_ptr(), data.as_ptr());

        // 3. Simulate DMA Read (peripheral reads from host memory)
        //
        // In Miri, we just check if we can read via the raw pointer while the
        // borrow is active.
        let val = unsafe { ptr::read(dma.as_ptr().add(1)) };
        assert_eq!(val, 20);

        // 4. Drop (Safe)
        drop(dma);
    }

    #[test]
    fn test_dma_slice_mut_write_cycle() {
        let fence = unsafe { MockFence::new() };

        let mut data = [0u8; 4];
        let data_ptr = data.as_mut_ptr();

        // 1. Create DmaSliceMut
        //
        // SAFETY: We call `restore_slice_ref` at the end.
        let dma = unsafe { DmaSliceMut::from_mut_slice_ref(&mut data, fence) };

        // 2. Verify basic pointer integrity
        assert_eq!(dma.as_mut_ptr(), data_ptr);
        assert_eq!(dma.len(), 4);

        // 3. Simulate DMA Write
        //
        // The peripheral writes `0xAA` to index 2.
        unsafe {
            simulate_dma_write(dma.as_mut_ptr(), 0xAA_u8, 2);
        }

        // 4. Restore
        //
        // SAFETY: DMA is "done".
        let restored_slice = unsafe { dma.restore_mut_slice_ref(fence) };

        // 5. Verify that the writes are reflected in the buffer:
        assert_eq!(restored_slice, &[0, 0, 0xAA, 0]);
    }

    #[test]
    fn test_dma_slice_mut_static() {
        let fence = unsafe { MockFence::new() };

        // Test specifically for the static constructor which is safe:
        static mut BUFFER: [u32; 2] = [1, 2];

        // 1. Create from static
        //
        // Note: access to static mut is unsafe, but the from_static_slice_ref call itself is safe
        let dma = DmaSliceMut::from_static_mut_slice_ref(unsafe { &mut *(&raw mut BUFFER) }, fence);

        // 2. Simulate DMA Write
        unsafe {
            simulate_dma_write(dma.as_mut_ptr(), 99u32, 0);
        }

        // 3. Restore
        let restored = unsafe { dma.restore_mut_slice_ref(fence) };

        assert_eq!(restored[0], 99);
        assert_eq!(restored[1], 2);
    }

    #[test]
    fn test_dma_sub_slice_immut() {
        let fence = unsafe { MockFence::new() };

        let data = [100u8, 101, 102, 103, 104];

        // Create a SubSlice with an active range of 1..=2
        let mut sub = SubSlice::new(&data);
        sub.slice(1..=2);

        let dma = DmaSubSlice::from_sub_slice(sub, fence);
        assert_eq!(dma.len(), 2);

        unsafe {
            assert_eq!(*dma.as_ptr(), 101);
            assert_eq!(*dma.as_ptr().add(1), 102);
        }
    }

    #[test]
    fn test_dma_sub_slice_mut_offset() {
        let fence = unsafe { MockFence::new() };

        let mut data = [0u64, 1, 2, 3, 4]; // u64 to test stride sizes > 1 byte

        // Create a SubSliceMut with an active range of 2..4
        let mut sub = SubSliceMut::new(&mut data);
        sub.slice(2..4);

        let dma = unsafe { DmaSubSliceMut::from_sub_slice_mut(sub, fence) };
        assert_eq!(dma.len(), 2);

        // Verify Pointer logic
        //
        // dma.as_ptr() should point to data[2]
        unsafe {
            // Write to index 0 of the DMA view (which is index 2 of underlying)
            simulate_dma_write(dma.as_mut_ptr(), 0xFF_FF_FF_FF_u64, 0);

            // Write to index 1 of the DMA view (index 3 of underlying)
            simulate_dma_write(dma.as_mut_ptr(), 0xEE_EE_EE_EE_u64, 1);
        }

        // 3. Restore
        let restored_sub = unsafe { dma.restore_sub_slice_mut(fence) };
        let full_slice = restored_sub.take();

        // 4. Validate
        assert_eq!(full_slice[0], 0); // Untouched
        assert_eq!(full_slice[1], 1); // Untouched
        assert_eq!(full_slice[2], 0xFF_FF_FF_FF); // Written
        assert_eq!(full_slice[3], 0xEE_EE_EE_EE); // Written
        assert_eq!(full_slice[4], 4); // Untouched
    }

    #[test]
    fn test_dma_sub_slice_mut_edge_cases() {
        let fence = unsafe { MockFence::new() };

        let mut data = [0u8; 10];
        let data_ptr = data.as_mut_ptr();

        // Case A: Empty Range
        {
            let mut sub = SubSliceMut::new(&mut data);
            sub.slice(5..5);

            let dma = unsafe { DmaSubSliceMut::from_sub_slice_mut(sub, fence) };
            assert_eq!(dma.len(), 0);

            // Verify we return the correct ptr, even if we shouldn't deref it
            assert_eq!(dma.as_mut_ptr(), data_ptr.wrapping_add(5));
            unsafe { dma.restore_sub_slice_mut(fence) };
        }

        // Case B: Range at exact end
        {
            let base_addr = data.as_ptr() as usize;

            let mut sub = SubSliceMut::new(&mut data);
            sub.slice(10..10); // End of buffer

            let dma = unsafe { DmaSubSliceMut::from_sub_slice_mut(sub, fence) };

            // Pointer should point one past the end of the array
            let ptr_addr = dma.as_mut_ptr() as usize;
            assert_eq!(ptr_addr, base_addr + 10);

            unsafe { dma.restore_sub_slice_mut(fence) };
        }

        // Case C: Range out of bounds (Should be clamped by implementation)
        {
            let mut sub = SubSliceMut::new(&mut data);
            sub.slice(8..15); // End is past 10

            let dma = unsafe { DmaSubSliceMut::from_sub_slice_mut(sub, fence) };

            // Length should be clamped to available (10 - 8 = 2)
            assert_eq!(dma.len(), 2);

            unsafe {
                simulate_dma_write(dma.as_mut_ptr(), 99u8, 0); // index 8
                simulate_dma_write(dma.as_mut_ptr(), 88u8, 1); // index 9
            }

            let res = unsafe { dma.restore_sub_slice_mut(fence) };
            let arr = res.take();
            assert_eq!(arr[8], 99);
            assert_eq!(arr[9], 88);
        }
    }
}
