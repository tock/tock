// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Google LLC 2024.

/// A custom never type because ! is not stable
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Never {}
impl Default for Never {
    fn default() -> Self {
        panic!()
    }
}
