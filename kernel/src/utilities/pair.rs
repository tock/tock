// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2025.

//! Support for pairs of heterogeneous values

use core::fmt::Debug;

/// A pair of heterogeneous values.
pub struct Pair<T, U>(T, U);

impl<T, U> Pair<T, U> {
    pub const fn new(first: T, second: U) -> Self {
        Self(first, second)
    }

    pub const fn as_first(&self) -> &T {
        &self.0
    }

    pub fn to_first(self) -> T {
        self.0
    }

    pub const fn as_second(&self) -> &U {
        &self.1
    }

    pub fn to_second(self) -> U {
        self.1
    }

    pub fn consume(self) -> (T, U) {
        (self.0, self.1)
    }
}

impl<T: Debug, U: Debug> Debug for Pair<T, U> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "Pair({:?}, {:?})",
            self.as_first(),
            self.as_second()
        )
    }
}
