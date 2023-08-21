// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Wrapper type for safe pointers to static memory.

use core::ops::Deref;
use core::ptr::NonNull;

/// A pointer to statically allocated mutable data such as memory mapped I/O
/// registers.
///
/// This is a simple wrapper around a raw pointer that encapsulates an unsafe
/// dereference in a safe manner. It serve the role of creating a `&'static T`
/// given a raw address and acts similarly to `extern` definitions, except
/// `StaticRef` is subject to module and crate boundaries, while `extern`
/// definitions can be imported anywhere.
///
/// Because this defers the actual dereference, this can be put in a `const`,
/// whereas `const I32_REF: &'static i32 = unsafe { &*(0x1000 as *const i32) };`
/// will always fail to compile since `0x1000` doesn't have an allocation at
/// compile time, even if it's known to be a valid MMIO address.
#[derive(Debug)]
pub struct StaticRef<T> {
    ptr: NonNull<T>,
}

impl<T> StaticRef<T> {
    /// Create a new `StaticRef` from a raw pointer
    ///
    /// ## Safety
    ///
    /// - `ptr` must be aligned, non-null, and dereferencable as `T`.
    /// - `*ptr` must be valid for the program duration.
    pub const unsafe fn new(ptr: *const T) -> StaticRef<T> {
        // SAFETY: `ptr` is non-null as promised by the caller.
        StaticRef {
            ptr: NonNull::new_unchecked(ptr.cast_mut()),
        }
    }
}

impl<T> Clone for StaticRef<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for StaticRef<T> {}

impl<T> Deref for StaticRef<T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY: `ptr` is aligned and dereferencable for the program
        // duration as promised by the caller of `StaticRef::new`.
        unsafe { self.ptr.as_ref() }
    }
}
