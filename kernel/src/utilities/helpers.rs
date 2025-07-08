// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Helper functions and macros.
//!
//! These are various utility functions and macros that are useful throughout
//! the Tock kernel and are provided here for convenience.
//!
//! The macros are exported through the top level of the `kernel` crate.

/// Create an object with the given capabilities.
///
/// ```
/// # use kernel::capabilities::{ProcessManagementCapability, MemoryAllocationCapability};
/// # use kernel::create_capability;
/// let process_mgmt_cap = create_capability!(ProcessManagementCapability);
/// let unified_cap = create_capability!(ProcessManagementCapability, MemoryAllocationCapability);
/// ```
///
/// This helper macro cannot be called from `#![forbid(unsafe_code)]` crates,
/// and is used by trusted code to generate a capability that it can either use
/// or pass to another module.
///
/// # Safety
///
/// This macro can only be used in a context that is allowed to use
/// `unsafe`. Specifically, an internal `allow(unsafe_code)` directive
/// will conflict with any `forbid(unsafe_code)` at the crate or block
/// level.
///
/// ```compile_fail
/// # use kernel::capabilities::ProcessManagementCapability;
/// # use kernel::create_capability;
/// #[forbid(unsafe_code)]
/// fn untrusted_fn() {
///     let process_mgmt_cap = create_capability!(ProcessManagementCapability);
/// }
/// ```
#[macro_export]
macro_rules! create_capability {
    ($($T:ty),+) => {{
        #[allow(unsafe_code)]
        struct Cap(());
        $(
            unsafe impl $T for Cap {}
        )*
        Cap(())
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

/// Compute a POSIX-style CRC32 checksum of a slice.
///
/// Online calculator: <https://crccalc.com/>
pub fn crc32_posix(b: &[u8]) -> u32 {
    let mut crc: u32 = 0;

    for c in b {
        crc ^= (*c as u32) << 24;

        for _i in 0..8 {
            if crc & (0b1 << 31) > 0 {
                crc = (crc << 1) ^ 0x04c11db7;
            } else {
                crc <<= 1;
            }
        }
    }
    !crc
}
