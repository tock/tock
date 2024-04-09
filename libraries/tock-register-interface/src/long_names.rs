// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.
// Copyright Google LLC 2024.

use core::marker::PhantomData;

pub struct Aliased<Read: RegisterLongName, Write: RegisterLongName> {
    _empty: Empty,
    _read: PhantomData<Read>,
    _write: PhantomData<Write>,
}

impl<Read: RegisterLongName, Write: RegisterLongName> LongNames for Aliased<Read, Write> {
    type Read = Read;
    type Write = Write;
}

enum Empty {}

/// Descriptive name for each register.
pub trait RegisterLongName {}

// Useful implementation for when no RegisterLongName is required
// (e.g. no fields need to be accessed, just the raw register values)
impl RegisterLongName for () {}

pub trait LongNames {
    type Read: RegisterLongName;
    type Write: RegisterLongName;
}

impl<RLN: RegisterLongName> LongNames for RLN {
    type Read = RLN;
    type Write = RLN;
}
