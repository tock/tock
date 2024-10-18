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
            unsafe {
                ptr::copy_nonoverlapping(src.as_ptr(), self.as_mut_ptr(), self.len());
            }
            Ok(())
        } else {
            Err(ErrorCode::SIZE)
        }
    }
}
