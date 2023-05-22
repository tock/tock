// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use crate::ErrorCode;
use core::ptr;

pub trait CopyOrErr {
    /// Copies a nonoverlapping slice from src to self. Returns Err(ErrorCode) if source and self
    /// are not the same length. This is a non-panicing version of slice::copy_from_slice.
    fn copy_from_slice_or_err(&mut self, src: &Self) -> Result<(), ErrorCode>;
}

impl CopyOrErr for [u8] {
    fn copy_from_slice_or_err(&mut self, src: &Self) -> Result<(), ErrorCode> {
        if self.len() == src.len() {
            // SAFETY: `self` is valid for `self.len()` elements by definition, and `src` was
            // checked to have the same length. The slices cannot overlap because
            // mutable references are exclusive.
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
            // SAFETY: `self` is valid for `self.len()` elements by definition, and `src` was
            // checked to have the same length. The slices cannot overlap because
            // mutable references are exclusive.
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
            // SAFETY: `self` is valid for `self.len()` elements by definition, and `src` was
            // checked to have the same length. The slices cannot overlap because
            // mutable references are exclusive.
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
            // SAFETY: `self` is valid for `self.len()` elements by definition, and `src` was
            // checked to have the same length. The slices cannot overlap because
            // mutable references are exclusive.
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
            // SAFETY: `self` is valid for `self.len()` elements by definition, and `src` was
            // checked to have the same length. The slices cannot overlap because
            // mutable references are exclusive.
            unsafe {
                ptr::copy_nonoverlapping(src.as_ptr(), self.as_mut_ptr(), self.len());
            }
            Ok(())
        } else {
            Err(ErrorCode::SIZE)
        }
    }
}
