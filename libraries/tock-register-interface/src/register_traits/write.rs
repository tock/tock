// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.
// Copyright Google LLC 2024.

use crate::fields::FieldValue;
use crate::{ArrayDataType, LocalRegisterCopy, Register, RegisterLongName, UIntLike};

/// A register that can safely be written to.
pub trait Write: Register {
    type LongName: RegisterLongName;

    /// Set the raw register value
    fn write(&self, value: Self::DataType)
    where
        Self::DataType: UIntLike;

    /// Write an array register without bounds checking. Instead of using
    /// `write_at_unchecked` directly, callers are encouraged to call
    /// `ArrayRegister::get()` to get an `ArrayElement` pointing at the
    /// register, then invoke `write` on that.
    ///
    /// # Safety
    /// `index` must be less than `Self::LEN`.
    unsafe fn write_at_unchecked(
        self,
        index: usize,
        value: <Self::DataType as ArrayDataType>::Element,
    ) where
        Self::DataType: ArrayDataType;

    /// Write the value of one or more fields, overwriting the other fields with zero
    fn write_field(&self, field: FieldValue<Self::DataType, Self::LongName>)
    where
        Self::DataType: UIntLike,
    {
        self.write(field.value);
    }

    /// Write the value of one or more fields, maintaining the value of unchanged fields via a
    /// provided original value, rather than a register read.
    fn modify_no_read(
        &self,
        original: LocalRegisterCopy<Self::DataType, Self::LongName>,
        field: FieldValue<Self::DataType, Self::LongName>,
    ) where
        Self::DataType: UIntLike,
    {
        self.write(field.modify(original.get()));
    }
}
