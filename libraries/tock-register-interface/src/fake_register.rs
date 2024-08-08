// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

use crate::{
    Access, ArrayDataType, DataType, NoAccess, OutOfBounds, Register, Safe, ScalarDataType, Unsafe,
    UnsafeRead,
};

#[allow(private_bounds)]
pub struct FakeRegister<Data: Copy, DT: DataType, Read: Access, Write: Access>
where
    (DT, Read): ReadFn,
    (DT, Write): WriteFn,
{
    data: Data,
    read_fn: <(DT, Read) as ReadFn>::Fn<Data>,
    _write_fn: <(DT, Write) as WriteFn>::Fn<Data>,
}

impl<Data: Copy, DT: DataType, Read: Access, Write: Access> Clone
    for FakeRegister<Data, DT, Read, Write>
where
    (DT, Read): ReadFn,
    (DT, Write): WriteFn,
{
    fn clone(&self) -> Self {
        *self
    }
}
impl<Data: Copy, DT: DataType, Read: Access, Write: Access> Copy
    for FakeRegister<Data, DT, Read, Write>
where
    (DT, Read): ReadFn,
    (DT, Write): WriteFn,
{
}

impl<Data: Copy, DT: DataType, Read: Access, Write: Access> Register
    for FakeRegister<Data, DT, Read, Write>
where
    (DT, Read): ReadFn,
    (DT, Write): WriteFn,
{
    type DataType = DT;
}

impl<Data: Copy, DT: DataType, Write: Access> UnsafeRead for FakeRegister<Data, DT, Unsafe, Write>
where
    (DT, Unsafe): ReadFn<Fn<Data> = unsafe fn(Data) -> DT::Value>,
    (DT, Write): WriteFn,
{
    unsafe fn read(self) -> DT::Value {
        // Safety: The caller has complied with this register's
        // hardware-specific safety invariants.
        unsafe { (self.read_fn)(self.data) }
    }

    unsafe fn read_at_unchecked(self, _index: usize) -> DT::Value
    where
        DT: ArrayDataType,
    {
        panic!("FakeRegister::unsafe_read_unchecked called on a scalar data type");
    }
}

trait ReadFn {
    type Fn<Data>: Copy;
}

impl<DT> ReadFn for (DT, NoAccess) {
    type Fn<Data> = ();
}

impl<S: ScalarDataType> ReadFn for (S, Safe) {
    type Fn<Data> = fn(Data) -> S::Value;
}

impl<S: ScalarDataType> ReadFn for (S, Unsafe) {
    /// # Safety
    /// The caller must comply with this register's hardware-specific safety
    /// requirements. Note that the fake version of this peripheral may or may
    /// not actually be unsafe to use -- but `FakeRegister` doesn't know that
    /// and therefore has to assume it is unsafe.
    type Fn<Data> = unsafe fn(Data) -> S::Value;
}

impl<S: ScalarDataType, const LEN: usize> ReadFn for ([S; LEN], Safe) {
    type Fn<Data> = fn(Data, usize) -> Option<S::Value>;
}

impl<S: ScalarDataType, const LEN: usize> ReadFn for ([S; LEN], Unsafe) {
    /// # Safety
    /// The caller must comply with this register's hardware-specific safety
    /// requirements. Note that the fake version of this peripheral may or may
    /// not actually be unsafe to use -- but `FakeRegister` doesn't know that
    /// and therefore has to assume it is unsafe.
    type Fn<Data> = unsafe fn(Data, usize) -> Option<S::Value>;
}

trait WriteFn {
    type Fn<Data>: Copy;
}

impl<DT> WriteFn for (DT, NoAccess) {
    type Fn<Data> = ();
}

impl<S: ScalarDataType> WriteFn for (S, Safe) {
    type Fn<Data> = fn(Data, S);
}

impl<S: ScalarDataType> WriteFn for (S, Unsafe) {
    /// # Safety
    /// The caller must comply with this register's hardware-specific safety
    /// requirements. Note that the fake version of this peripheral may or may
    /// not actually be unsafe to use -- but `FakeRegister` doesn't know that
    /// and therefore has to assume it is unsafe.
    type Fn<Data> = unsafe fn(Data, S);
}

impl<S: ScalarDataType, const LEN: usize> WriteFn for ([S; LEN], Safe) {
    type Fn<Data> = fn(Data, usize, S::Value) -> Result<(), OutOfBounds>;
}

impl<S: ScalarDataType, const LEN: usize> WriteFn for ([S; LEN], Unsafe) {
    /// # Safety
    /// The caller must comply with this register's hardware-specific safety
    /// requirements. Note that the fake version of this peripheral may or may
    /// not actually be unsafe to use -- but `FakeRegister` doesn't know that
    /// and therefore has to assume it is unsafe.
    type Fn<Data> = unsafe fn(Data, usize, S::Value) -> Result<(), OutOfBounds>;
}
