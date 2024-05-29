// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

use crate::UIntLike;
use core::marker::PhantomData;

/// Descriptive name for each register.
pub trait RegisterLongName {}

// Useful implementation for when no RegisterLongName is required
// (e.g. no fields need to be accessed, just the raw register values)
impl RegisterLongName for () {}

/// A data type that may be specified for a register. Every `DataType` is either
/// a `ScalarDataType` or an `ArrayDataType`.
///
/// In the peripheral declaration syntax, the data types are as follows:
/// ```
/// use tock_registers::{Aliased, peripheral, register_bitfields};
/// peripheral! {
///     Foo {
///         0 => a: u32 { Read },
///         //      ^^^
///
///         4 => b: [u32; 2] { Read },
///         //      ^^^^^^^^
///
///         12 => c: Ctrl::Register { Write },
///         //       ^^^^^^^^^^^^^^
///
///         16 => d: [Ctrl::Register; 2] { Write },
///         //       ^^^^^^^^^^^^^^^^^^^
///
///         24 => e: Aliased<Ctrl::Register, Status::Register> { Read, Write },
///         //       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
///
///         32 => f: [Aliased<Ctrl::Register, Status::Register>; 2] { Read, Write },
///         //       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
///     }
/// }
///
/// register_bitfields![u32,
///     Ctrl [INT OFFSET(2) NUMBITS(1) []],
///     Status [TXCOMPLETE  OFFSET(0) NUMBITS(1) []]
/// ];
/// ```
pub trait DataType {
    /// The type of data that is read from or written to this register. This is
    /// always some form of primitive.
    type Value: UIntLike;

    /// Note: For most uses, prefer to use `ArrayDataType::LEN` rather than
    /// `NUM_VALUES`.
    ///
    /// This is the number of values of type `Value` contained in this register.
    /// Used to compute the size of the register (for verifying register
    /// offsets).
    const NUM_VALUES: usize;
}

impl<U: UIntLike> DataType for U {
    const NUM_VALUES: usize = 1;
    type Value = U;
}

impl<E: ScalarDataType, const LEN: usize> DataType for [E; LEN] {
    const NUM_VALUES: usize = LEN;
    type Value = E::Value;
}

/// A data type for a register with a single value.
pub trait ScalarDataType: DataType {
    /// The bitfield used when data is read from this register.
    type Read: RegisterLongName;

    /// The bitfield used when data is written to this register.
    type Write: RegisterLongName;
}

/// A data type for an array-typed register (which generally has more than one
/// value).
pub trait ArrayDataType: DataType {
    /// The type of a single member of the array. E.g. if the array is a
    /// `[u32; 4]`, `Element` will be `u32`.
    type Element: ScalarDataType<Value = Self::Value>;

    /// The number of elements in the array.
    const LEN: usize;
}

impl<E: ScalarDataType, const LEN: usize> ArrayDataType for [E; LEN] {
    type Element = E;
    const LEN: usize = LEN;
}

/// Data type for a register than has a different meaning when read from or
/// written to. `Read` and `Write` are expected to be different bitfields with
/// the same value type.
pub struct Aliased<Read: ScalarDataType, Write: ScalarDataType> {
    _empty: Empty,
    _read: PhantomData<Read>,
    _write: PhantomData<Write>,
}

impl<U: UIntLike, Read: ScalarDataType<Value = U>, Write: ScalarDataType<Value = U>> DataType
    for Aliased<Read, Write>
{
    type Value = U;
    const NUM_VALUES: usize = 1;
}

impl<U: UIntLike, Read: ScalarDataType<Value = U>, Write: ScalarDataType<Value = U>> ScalarDataType
    for Aliased<Read, Write>
{
    type Read = Read::Read;
    type Write = Write::Write;
}

// Exists to make Aliased an uninhabited type.
enum Empty {}
