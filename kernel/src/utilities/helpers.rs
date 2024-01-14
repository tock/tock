// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Helper macros.

/// Create an object with the given capability.
///
/// ```ignore
/// use kernel::capabilities::ProcessManagementCapability;
/// use kernel;
///
/// let process_mgmt_cap = create_capability!(ProcessManagementCapability);
/// ```
///
/// This helper macro cannot be called from `#![forbid(unsafe_code)]` crates,
/// and is used by trusted code to generate a capability that it can either use
/// or pass to another module.
#[macro_export]
macro_rules! create_capability {
    ($T:ty $(,)?) => {{
        struct Cap;
        #[allow(unsafe_code)]
        unsafe impl $T for Cap {}
        Cap
    }};
}

/// Count the number of passed expressions.
///
/// Useful for constructing variable sized arrays in other macros.
/// Taken from the Little Book of Rust Macros.
///
/// ```ignore
/// use kernel:count_expressions;
///
/// let count: usize = count_expressions!(1+2, 3+4);
/// ```
#[macro_export]
macro_rules! count_expressions {
    () => (0usize);
    ($head:expr $(,)?) => (1usize);
    ($head:expr, $($tail:expr),* $(,)?) => (1usize + count_expressions!($($tail),*));
}

/// A very simple hashing algorithm "AddHash".
///
/// This just adds the ASCII character values from a string to create a 32 bit
/// hash value. To make different orders of the same characters more likely to
/// hash to different values, the characters are shifted left incrementally
/// before summing.
pub fn addhash_str(s: &'static str) -> u32 {
    let mut sum = 0_u32;
    for (i, c) in s.chars().enumerate() {
        sum += ((c as u8) as u32) << (i % 25);
    }
    sum
}
