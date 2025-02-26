// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Helper functions for converting values and data types.

/// Helper function to convert create a full usize value from two 32-bit usize
/// values.
///
/// In C this would look like:
///
/// ```c
/// size_t v = (hi << 32) | (uint32_t) lo;
/// ```
///
/// This is useful when passing a machine-sized value (i.e. a `size_t`) via the
/// system call interface in two 32-bit usize values. On a 32-bit machine this
/// essentially has no effect; the full value is stored in the `lo` usize. On a
/// 64-bit machine, this creates a usize by concatenating the hi and lo 32-bit
/// values.
///
/// TODO
/// ----
///
/// This can be more succinctly implemented using
/// [`unbounded_shl()`](https://doc.rust-lang.org/stable/std/primitive.usize.html#method.unbounded_shl).
/// However, that method is currently a nightly-only feature.
#[inline]
pub const fn usize32_to_usize(lo: usize, hi: usize) -> usize {
    if usize::BITS <= 32 {
        // Just return the lo value since it has the bits we need.
        lo
    } else {
        // Create a 64-bit value.
        (lo & 0xFFFFFFFF) | (hi << 32)
    }
}
