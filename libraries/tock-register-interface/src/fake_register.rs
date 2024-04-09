// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

use crate::{
    Access, ArrayDataType, LongNames, NoAccess, OutOfBounds, Register, Safe, UIntLike, Unsafe,
    UnsafeRead,
};
use core::marker::PhantomData;

#[allow(private_bounds)]
pub struct FakeRegister<Data: Copy, DT, LN: LongNames, Read: Access, Write: Access>
where
    (DT, Read): ReadFn,
    (DT, Write): WriteFn,
{
    data: Data,
    _phantom: PhantomData<LN>,
    read_fn: <(DT, Read) as ReadFn>::Fn<Data>,
    _write_fn: <(DT, Write) as WriteFn>::Fn<Data>,
}

impl<Data: Copy, DT, LN: LongNames, Read: Access, Write: Access> Clone
    for FakeRegister<Data, DT, LN, Read, Write>
where
    (DT, Read): ReadFn,
    (DT, Write): WriteFn,
{
    fn clone(&self) -> Self {
        *self
    }
}
impl<Data: Copy, DT, LN: LongNames, Read: Access, Write: Access> Copy
    for FakeRegister<Data, DT, LN, Read, Write>
where
    (DT, Read): ReadFn,
    (DT, Write): WriteFn,
{
}

impl<Data: Copy, DT, LN: LongNames, Read: Access, Write: Access> Register
    for FakeRegister<Data, DT, LN, Read, Write>
where
    (DT, Read): ReadFn,
    (DT, Write): WriteFn,
{
    type DataType = DT;
}

impl<Data: Copy, U: UIntLike, LN: LongNames, Write: Access> UnsafeRead
    for FakeRegister<Data, U, LN, Unsafe, Write>
where
    (U, Unsafe): ReadFn<Fn<Data> = unsafe fn(Data) -> U>,
    (U, Write): WriteFn,
{
    unsafe fn read(self) -> U {
        // Safety: The caller has complied with this register's
        // hardware-specific safety invariants.
        unsafe { (self.read_fn)(self.data) }
    }

    unsafe fn read_at_unchecked(self, _index: usize) -> <U as ArrayDataType>::Element
    where
        U: ArrayDataType,
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

impl<U: UIntLike> ReadFn for (U, Safe) {
    type Fn<Data> = fn(Data) -> U;
}

impl<U: UIntLike> ReadFn for (U, Unsafe) {
    /// # Safety
    /// The caller must comply with this register's hardware-specific safety
    /// requirements. Note that the fake version of this peripheral may or may
    /// not actually be unsafe to use -- but `FakeRegister` doesn't know that
    /// and therefore has to assume it is unsafe.
    type Fn<Data> = unsafe fn(Data) -> U;
}

impl<U: UIntLike, const LEN: usize> ReadFn for ([U; LEN], Safe) {
    type Fn<Data> = fn(Data, usize) -> Option<U>;
}

impl<U: UIntLike, const LEN: usize> ReadFn for ([U; LEN], Unsafe) {
    /// # Safety
    /// The caller must comply with this register's hardware-specific safety
    /// requirements. Note that the fake version of this peripheral may or may
    /// not actually be unsafe to use -- but `FakeRegister` doesn't know that
    /// and therefore has to assume it is unsafe.
    type Fn<Data> = unsafe fn(Data, usize) -> Option<U>;
}

trait WriteFn {
    type Fn<Data>: Copy;
}

impl<DT> WriteFn for (DT, NoAccess) {
    type Fn<Data> = ();
}

impl<U: UIntLike> WriteFn for (U, Safe) {
    type Fn<Data> = fn(Data, U);
}

impl<U: UIntLike> WriteFn for (U, Unsafe) {
    /// # Safety
    /// The caller must comply with this register's hardware-specific safety
    /// requirements. Note that the fake version of this peripheral may or may
    /// not actually be unsafe to use -- but `FakeRegister` doesn't know that
    /// and therefore has to assume it is unsafe.
    type Fn<Data> = unsafe fn(Data, U);
}

impl<U: UIntLike, const LEN: usize> WriteFn for ([U; LEN], Safe) {
    type Fn<Data> = fn(Data, usize, U) -> Result<(), OutOfBounds>;
}

impl<U: UIntLike, const LEN: usize> WriteFn for ([U; LEN], Unsafe) {
    /// # Safety
    /// The caller must comply with this register's hardware-specific safety
    /// requirements. Note that the fake version of this peripheral may or may
    /// not actually be unsafe to use -- but `FakeRegister` doesn't know that
    /// and therefore has to assume it is unsafe.
    type Fn<Data> = unsafe fn(Data, usize, U) -> Result<(), OutOfBounds>;
}
