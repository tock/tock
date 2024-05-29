// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

use crate::{ArrayDataType, DataType, ScalarDataType};

mod read;
mod write;

pub use read::Read;
pub use write::Write;

/// Trait implemented by all register types. Note that the Register
/// implementations only point to the real register (e.g. they are pointer or
/// reference types), hence why they may be copied.
///
/// Registers are further divided into `ArrayRegister`s and `ScalarRegister`s.
pub trait Register: Copy {
    type DataType: DataType;
}

/// A register that can be read, but which is not memory-safe to read.
pub trait UnsafeRead: Register {
    /// # Safety
    /// Reading this register has hardware-specific safety requirements which
    /// the caller must comply with.
    unsafe fn read(self) -> <Self::DataType as DataType>::Value
    where
        Self::DataType: ScalarDataType;

    /// Read from an unsafe array register. Instead of using `read_at_unchecked`
    /// directly, callers are encouraged to call `get()` to get an
    /// `ArrayElement` pointing at the register, then invoke `read` on that.
    ///
    /// # Safety
    /// `index` must be less than `Self::LEN`.
    /// Reading this register has hardware-specific safety requirements which
    /// the caller must comply with.
    unsafe fn read_at_unchecked(self, index: usize) -> <Self::DataType as DataType>::Value
    where
        Self::DataType: ArrayDataType;
}

/// A regsiter that can be written, but which is not memory-safe to write.
pub trait UnsafeWrite: Register {
    /// # Safety
    /// Writing this register has hardware-specific safety requirements which
    /// the caller must comply with.
    unsafe fn write(self, value: <Self::DataType as DataType>::Value)
    where
        Self::DataType: ScalarDataType;

    /// Write to an unsafe array register. Instead of using `write_at_unchecked`
    /// directly, callers are encouraged to call `get()` to get an
    /// `ArrayElement` pointing at the register, then invoke `read` on that.
    ///
    /// # Safety
    /// `index` must be less than `Self::LEN`.
    /// Writing this register has hardware-specific safety requirements which
    /// the caller must comply with.
    unsafe fn write_at_unchecked(self, index: usize, value: <Self::DataType as DataType>::Value)
    where
        Self::DataType: ArrayDataType;
}
