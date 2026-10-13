// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Helper functions for copying buffers.
//!
//! This utility provides an implementation of the standard Rust
//! [`slice::copy_from_slice()`] method that cannot panic. This method is
//! provided through the [`CopyOrErr`] trait.
//!
//! This functionality is currently provided for the following types:
//! - `u8`
//! - `u16`
//! - `u32`
//! - `u64`
//! - `usize`

use crate::ErrorCode;
use core::ptr;

/// Interface for copying buffers that cannot panic.
pub trait CopyOrErr {
    /// Copy a non-overlapping slice from `src` to `self`.
    ///
    /// This is a non-panicking version of [`slice::copy_from_slice`].
    ///
    /// Returns `Err(ErrorCode)` if `src` and `self` are not the same length.
    fn copy_from_slice_or_err(&mut self, src: &Self) -> Result<(), ErrorCode>;
}

impl CopyOrErr for [u8] {
    fn copy_from_slice_or_err(&mut self, src: &Self) -> Result<(), ErrorCode> {
        if self.len() == src.len() {
            // SAFETY: `self` is valid for `self.len()` elements by definition,
            // and `src` was checked to have the same length. The slices cannot
            // overlap because mutable references are exclusive.
            //
            // SAFETY REQUIREMENTS:
            // - calling unsafe function `copy_nonoverlapping`
            //   - https://doc.rust-lang.org/core/ptr/fn.copy_nonoverlapping.html
            //   - Behavior is undefined if any of the following conditions are violated:
            //
            //     - src must be valid for reads of count * size_of::<T>() bytes or that number must be 0.
            //     - dst must be valid for writes of count * size_of::<T>() bytes or that number must be 0.
            //     - Both src and dst must be properly aligned.
            //     - The region of memory beginning at src with a size of count * size_of::<T>() bytes must not overlap with the region of memory beginning at dst with the same size.
            //
            //         Like read, copy_nonoverlapping creates a bitwise copy of T, regardless of whether T is Copy. If T is not Copy, using both the values in the region beginning at *src and the region beginning at *dst can violate memory safety.
            //
            //         Note that even if the effectively copied size (count * size_of::<T>()) is 0, the pointers must be properly aligned.
            unsafe {
                ptr::copy_nonoverlapping(src.as_ptr(), self.as_mut_ptr(), self.len());
            }
            Ok(())
        } else {
            Err(ErrorCode::SIZE)
        }
    }
}

impl CopyOrErr for [u16] {
    fn copy_from_slice_or_err(&mut self, src: &Self) -> Result<(), ErrorCode> {
        if self.len() == src.len() {
            // SAFETY: `self` is valid for `self.len()` elements by definition,
            // and `src` was checked to have the same length. The slices cannot
            // overlap because mutable references are exclusive.
            //
            // SAFETY REQUIREMENTS:
            // - calling unsafe function `copy_nonoverlapping`
            //   - https://doc.rust-lang.org/core/ptr/fn.copy_nonoverlapping.html
            //   - Behavior is undefined if any of the following conditions are violated:
            //
            //     - src must be valid for reads of count * size_of::<T>() bytes or that number must be 0.
            //     - dst must be valid for writes of count * size_of::<T>() bytes or that number must be 0.
            //     - Both src and dst must be properly aligned.
            //     - The region of memory beginning at src with a size of count * size_of::<T>() bytes must not overlap with the region of memory beginning at dst with the same size.
            //
            //         Like read, copy_nonoverlapping creates a bitwise copy of T, regardless of whether T is Copy. If T is not Copy, using both the values in the region beginning at *src and the region beginning at *dst can violate memory safety.
            //
            //         Note that even if the effectively copied size (count * size_of::<T>()) is 0, the pointers must be properly aligned.
            unsafe {
                ptr::copy_nonoverlapping(src.as_ptr(), self.as_mut_ptr(), self.len());
            }
            Ok(())
        } else {
            Err(ErrorCode::SIZE)
        }
    }
}

impl CopyOrErr for [u32] {
    fn copy_from_slice_or_err(&mut self, src: &Self) -> Result<(), ErrorCode> {
        if self.len() == src.len() {
            // SAFETY: `self` is valid for `self.len()` elements by definition,
            // and `src` was checked to have the same length. The slices cannot
            // overlap because mutable references are exclusive.
            //
            // SAFETY REQUIREMENTS:
            // - calling unsafe function `copy_nonoverlapping`
            //   - https://doc.rust-lang.org/core/ptr/fn.copy_nonoverlapping.html
            //   - Behavior is undefined if any of the following conditions are violated:
            //
            //     - src must be valid for reads of count * size_of::<T>() bytes or that number must be 0.
            //     - dst must be valid for writes of count * size_of::<T>() bytes or that number must be 0.
            //     - Both src and dst must be properly aligned.
            //     - The region of memory beginning at src with a size of count * size_of::<T>() bytes must not overlap with the region of memory beginning at dst with the same size.
            //
            //         Like read, copy_nonoverlapping creates a bitwise copy of T, regardless of whether T is Copy. If T is not Copy, using both the values in the region beginning at *src and the region beginning at *dst can violate memory safety.
            //
            //         Note that even if the effectively copied size (count * size_of::<T>()) is 0, the pointers must be properly aligned.
            unsafe {
                ptr::copy_nonoverlapping(src.as_ptr(), self.as_mut_ptr(), self.len());
            }
            Ok(())
        } else {
            Err(ErrorCode::SIZE)
        }
    }
}

impl CopyOrErr for [u64] {
    fn copy_from_slice_or_err(&mut self, src: &Self) -> Result<(), ErrorCode> {
        if self.len() == src.len() {
            // SAFETY: `self` is valid for `self.len()` elements by definition,
            // and `src` was checked to have the same length. The slices cannot
            // overlap because mutable references are exclusive.
            //
            // SAFETY REQUIREMENTS:
            // - calling unsafe function `copy_nonoverlapping`
            //   - https://doc.rust-lang.org/core/ptr/fn.copy_nonoverlapping.html
            //   - Behavior is undefined if any of the following conditions are violated:
            //
            //     - src must be valid for reads of count * size_of::<T>() bytes or that number must be 0.
            //     - dst must be valid for writes of count * size_of::<T>() bytes or that number must be 0.
            //     - Both src and dst must be properly aligned.
            //     - The region of memory beginning at src with a size of count * size_of::<T>() bytes must not overlap with the region of memory beginning at dst with the same size.
            //
            //         Like read, copy_nonoverlapping creates a bitwise copy of T, regardless of whether T is Copy. If T is not Copy, using both the values in the region beginning at *src and the region beginning at *dst can violate memory safety.
            //
            //         Note that even if the effectively copied size (count * size_of::<T>()) is 0, the pointers must be properly aligned.
            unsafe {
                ptr::copy_nonoverlapping(src.as_ptr(), self.as_mut_ptr(), self.len());
            }
            Ok(())
        } else {
            Err(ErrorCode::SIZE)
        }
    }
}

impl CopyOrErr for [usize] {
    fn copy_from_slice_or_err(&mut self, src: &Self) -> Result<(), ErrorCode> {
        if self.len() == src.len() {
            // SAFETY: `self` is valid for `self.len()` elements by definition,
            // and `src` was checked to have the same length. The slices cannot
            // overlap because mutable references are exclusive.
            //
            // SAFETY REQUIREMENTS:
            // - calling unsafe function `copy_nonoverlapping`
            //   - https://doc.rust-lang.org/core/ptr/fn.copy_nonoverlapping.html
            //   - Behavior is undefined if any of the following conditions are violated:
            //
            //     - src must be valid for reads of count * size_of::<T>() bytes or that number must be 0.
            //     - dst must be valid for writes of count * size_of::<T>() bytes or that number must be 0.
            //     - Both src and dst must be properly aligned.
            //     - The region of memory beginning at src with a size of count * size_of::<T>() bytes must not overlap with the region of memory beginning at dst with the same size.
            //
            //         Like read, copy_nonoverlapping creates a bitwise copy of T, regardless of whether T is Copy. If T is not Copy, using both the values in the region beginning at *src and the region beginning at *dst can violate memory safety.
            //
            //         Note that even if the effectively copied size (count * size_of::<T>()) is 0, the pointers must be properly aligned.
            unsafe {
                ptr::copy_nonoverlapping(src.as_ptr(), self.as_mut_ptr(), self.len());
            }
            Ok(())
        } else {
            Err(ErrorCode::SIZE)
        }
    }
}
