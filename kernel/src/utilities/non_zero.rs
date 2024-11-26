// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2025.

//! Helper macro to create a NonZero value known at compile time

#[macro_export]
macro_rules! non_zero {
    ($expression:expr) => {
        const { NonZero::new($expression).unwrap() }
    };
    ($expression:expr, $type:ty) => {
        const { NonZero::<$type>::new($expression).unwrap() }
    };
}
