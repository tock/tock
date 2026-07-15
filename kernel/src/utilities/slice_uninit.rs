// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

/// Convert a `&mut [T]` to a `&mut [MaybeUninit<T>]`.
///
/// This is a safe operation, but as of June 2026, there is no safe API in the
/// standard library to do this. So, we provide our own.
pub fn mut_slice_as_maybeuninit<T>(buffer: &mut [T]) -> &mut [core::mem::MaybeUninit<T>] {
    // # Safety
    //
    // `MaybeUninit<T>` has the same size and alignment as `T`.
    let maybeuninit_buf: &mut [core::mem::MaybeUninit<T>] =
        unsafe { core::slice::from_raw_parts_mut(buffer.as_mut_ptr().cast(), buffer.len()) };
    maybeuninit_buf
}
