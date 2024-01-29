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

/// Compute the CRC-32B checksum of a string.
///
/// Implementation based on https://lxp32.github.io/docs/a-simple-example-crc32-calculation/.
/// Online calculator here: https://md5calc.com/hash/crc32b
pub fn crc32b_str(s: &'static str) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;

    for c in s.chars() {
        let mut c: u8 = c as u8;

        for _i in 0..8 {
            let b: u32 = ((c as u32) ^ crc) & 0x1;
            crc >>= 1;
            if b != 0 {
                crc ^= 0xEDB88320;
            }
            c >>= 1;
        }
    }
    !crc
}
