// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

/// Create a raw slice from within a raw slice.
///
/// This creates the sub-slice `slice[start..end]` as a `*const [u8]`. This
/// does NOT create any intermediate Rust references.
///
/// Returns `None` if the requested range is out of bounds or the requested
/// slice cannot be created.
pub fn slice_ptr_subslice(slice: *const [u8], start: usize, end: usize) -> Option<*const [u8]> {
    // Check that the requested subslice is within the existing slice.
    if start > end || end > slice.len() {
        return None;
    }

    // Get a pointer to the beginning of the slice.
    let slice_ptr: *const u8 = slice.cast::<u8>();

    // We know that the new slice length will fit in the current slice.
    let new_slice_length = end - start;

    // Short circuit if start == 0 to avoid any unsafe operation.
    if start == 0 {
        Some(core::ptr::slice_from_raw_parts(slice_ptr, new_slice_length))
    } else {
        // Move the pointer up to the start location.
        //
        // SAFETY: Because of our initial check of `start` and `end`, we know
        // we are either keeping the slice the same size or making it
        // smaller. Therefore, because the original slice exists, we know the
        // offset will fit in a `isize` and the resulting size will fit in a
        // `usize`. And, this new slice must then be from the same
        // allocation.
        let new_slice_ptr = unsafe { slice_ptr.add(start) };

        // Create the new `*const [u8]`.
        Some(core::ptr::slice_from_raw_parts(
            new_slice_ptr,
            new_slice_length,
        ))
    }
}
