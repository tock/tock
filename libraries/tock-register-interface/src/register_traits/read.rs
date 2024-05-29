// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.
// Copyright Google LLC 2024.

use crate::fields::{Field, FieldValue, TryFromValue};
use crate::{ArrayDataType, DataType, LocalRegisterCopy, Register, ScalarDataType};

/// A register that can be safely read.
pub trait Read: Register {
    /// Get the raw register value.
    fn read(&self) -> <Self::DataType as DataType>::Value
    where
        Self::DataType: ScalarDataType;

    /// Read an array register without bounds checking. Instead of using
    /// `read_at_unchecked` directly, callers are encouraged to call
    /// `ArrayRegister::get()` to get an `ArrayElement` pointing at the
    /// register, then invoke `read` on that.
    ///
    /// # Safety
    /// `index` must be less than `Self::LEN`.
    unsafe fn read_at_unchecked(self, index: usize) -> <Self::DataType as DataType>::Value
    where
        Self::DataType: ArrayDataType;

    /// Read the value of the given field.
    fn read_field(
        &self,
        field: Field<<Self::DataType as DataType>::Value, <Self::DataType as ScalarDataType>::Read>,
    ) -> <Self::DataType as DataType>::Value
    where
        Self::DataType: ScalarDataType,
    {
        field.read(self.read())
    }

    /// Set the raw register value
    ///
    /// The [`register_bitfields!`](crate::register_bitfields) macro will
    /// generate an enum containing the various named field variants and
    /// implementing the required [`TryFromValue`] trait. It is accessible as
    /// `$REGISTER_NAME::$FIELD_NAME::Value`.
    ///
    /// This method can be useful to symbolically represent read register field
    /// states throughout the codebase and to enforce exhaustive matches over
    /// all defined valid register field values.
    fn read_as_enum<E: TryFromValue<<Self::DataType as DataType>::Value, EnumType = E>>(
        &self,
        field: Field<<Self::DataType as DataType>::Value, <Self::DataType as ScalarDataType>::Read>,
    ) -> Option<E>
    where
        Self::DataType: ScalarDataType,
    {
        field.read_as_enum(self.read())
    }

    /// Make a local copy of the register
    fn extract(
        &self,
    ) -> LocalRegisterCopy<
        <Self::DataType as DataType>::Value,
        <Self::DataType as ScalarDataType>::Read,
    >
    where
        Self::DataType: ScalarDataType,
    {
        LocalRegisterCopy::new(self.read())
    }

    /// Check if one or more bits in a field are set
    fn is_set(
        &self,
        field: Field<<Self::DataType as DataType>::Value, <Self::DataType as ScalarDataType>::Read>,
    ) -> bool
    where
        Self::DataType: ScalarDataType,
    {
        field.is_set(self.read())
    }

    /// Check if any bits corresponding to the mask in the passed `FieldValue` are set.
    /// This function is identical to `is_set()` but operates on a `FieldValue` rather
    /// than a `Field`, allowing for checking if any bits are set across multiple,
    /// non-contiguous portions of a bitfield.
    fn any_matching_bits_set(&self, field: ReadFieldValue<Self>) -> bool
    where
        Self::DataType: ScalarDataType,
    {
        field.any_matching_bits_set(self.read())
    }

    /// Check if all specified parts of a field match
    fn matches_all(&self, field: ReadFieldValue<Self>) -> bool
    where
        Self::DataType: ScalarDataType,
    {
        field.matches_all(self.read())
    }

    /// Check if any of the passed parts of a field exactly match the contained
    /// value. This allows for matching on unset bits, or matching on specific values
    /// in multi-bit fields.
    fn matches_any(&self, fields: &[ReadFieldValue<Self>]) -> bool
    where
        Self::DataType: ScalarDataType,
    {
        fields
            .iter()
            .any(|field| self.read() & field.mask() == field.value)
    }
}

type ReadFieldValue<R> = FieldValue<
    <<R as Register>::DataType as DataType>::Value,
    <<R as Register>::DataType as ScalarDataType>::Read,
>;
