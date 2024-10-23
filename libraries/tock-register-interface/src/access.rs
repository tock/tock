// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

pub trait Access: private::Sealed {}

pub enum NoAccess {}
impl Access for NoAccess {}
impl private::Sealed for NoAccess {}

pub enum Safe {}
impl Access for Safe {}
impl private::Sealed for Safe {}

pub enum Unsafe {}
impl Access for Unsafe {}
impl private::Sealed for Unsafe {}

mod private {
    pub trait Sealed {}
}
