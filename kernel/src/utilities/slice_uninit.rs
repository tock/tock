// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

/// Convert a `&mut [T]` to a `&mut [MaybeUninit<T>]`.
///
/// This is a safe operation, but as of June 2026, there is no safe API in the
/// standard library to do this. So, we provide our own.
pub fn mut_slice_as_maybeuninit<T>(buffer: &mut [T]) -> &mut [core::mem::MaybeUninit<T>] {
    // SAFETY: `MaybeUninit<T>` has the same size and alignment as `T`.
    let maybeuninit_buf: &mut [core::mem::MaybeUninit<T>] =
        //
        // SAFETY REQUIREMENTS:
        // - calling unsafe function `from_raw_parts_mut`
        //   - https://doc.rust-lang.org/core/slice/fn.from_raw_parts_mut.html
        //   - Behavior is undefined if any of the following conditions are violated:
        //
        //     - data must be non-null, valid for both reads and writes for len * size_of::<T>() many bytes, and it must be properly aligned. This means in particular:
        //
        //         - The entire memory range of this slice must be contained within a single allocation! Slices can never span across multiple allocations.
        //         - data must be non-null and aligned even for zero-length slices or slices of ZSTs. One reason for this is that enum layout optimizations may rely on references (including slices of any length) being aligned and non-null to distinguish them from other data. You can obtain a pointer that is usable as data for zero-length slices using NonNull::dangling().
        //     - data must point to len consecutive properly initialized values of type T.
        //     - The memory referenced by the returned slice must not be accessed through any other pointer (not derived from the return value) for the duration of lifetime 'a. Both read and write accesses are forbidden.
        //     - The total size len * size_of::<T>() of the slice must be no larger than isize::MAX, and adding that size to data must not “wrap around” the address space. See the safety documentation of pointer::offset.
        unsafe { core::slice::from_raw_parts_mut(buffer.as_mut_ptr().cast(), buffer.len()) };
    maybeuninit_buf
}
