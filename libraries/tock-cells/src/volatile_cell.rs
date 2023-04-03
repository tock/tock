// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

// This file was vendored into Tock. It comes from
// https://github.com/japaric/vcell, commit
// ef556474203e93b3f45f0f8cd8dea3210aa7f844, path src/lib.rs.

//! A type for accessing MMIO registers. `VolatileCell` is just like [`Cell`]
//! but with [volatile] read / write operations
//!
//! [`Cell`]: https://doc.rust-lang.org/std/cell/struct.Cell.html
//! [volatile]: https://doc.rust-lang.org/std/ptr/fn.read_volatile.html

#![deny(missing_docs)]
#![deny(warnings)]

use core::cell::UnsafeCell;
use core::ptr;

/// `VolatileCell` provides a wrapper around unsafe volatile pointer reads and
/// writes. This is particularly useful for accessing microcontroller registers
/// by (unsafely) casting a pointer to the register into a `VolatileCell`.
///
/// ```
/// use tock_cells::volatile_cell::VolatileCell;
/// let myptr: *const usize = 0xdeadbeef as *const usize;
/// let myregister: &VolatileCell<usize> = unsafe { core::mem::transmute(myptr) };
/// ```
#[derive(Default)]
#[repr(transparent)]
pub struct VolatileCell<T> {
    value: UnsafeCell<T>,
}

impl<T> VolatileCell<T> {
    /// Creates a new `VolatileCell` containing the given value
    pub const fn new(value: T) -> Self {
        VolatileCell {
            value: UnsafeCell::new(value),
        }
    }

    /// Performs a memory read and returns a copy of the value represented by
    /// the cell.
    ///
    /// # Side-Effects
    ///
    /// `get` _always_ performs a memory read on the underlying location. If
    /// this location is a memory-mapped I/O register, the side-effects of
    /// performing the read are register-specific.
    ///
    /// # Examples
    ///
    /// ```
    /// use tock_cells::volatile_cell::VolatileCell;
    ///
    /// let vc = VolatileCell::new(5);
    /// let five = vc.get();
    /// ```
    #[inline(always)]
    pub fn get(&self) -> T
    where
        T: Copy,
    {
        unsafe { ptr::read_volatile(self.value.get()) }
    }

    /// Performs a memory write with the provided value.
    ///
    /// # Side-Effects
    ///
    /// `set` _always_ performs a memory write on the underlying location. If
    /// this location is a memory-mapped I/O register, the side-effects of
    /// performing the write are register-specific.
    ///
    /// # Examples
    ///
    /// ```
    /// use tock_cells::volatile_cell::VolatileCell;
    ///
    /// let vc = VolatileCell::new(123);
    /// vc.set(432);
    /// ```
    #[inline(always)]
    pub fn set(&self, value: T)
    where
        T: Copy,
    {
        unsafe { ptr::write_volatile(self.value.get(), value) }
    }
}

// NOTE implicit because of `UnsafeCell`
// unsafe impl<T> !Sync for VolatileCell<T> {}
