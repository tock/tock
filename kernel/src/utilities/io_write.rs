// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Re-implementation of `std:io` traits not available in `no_std`.

use crate::collections::ring_buffer::RingBuffer;

/// Implementation of `std::io::Write` for `no_std`.
///
/// This takes bytes instead of a string (contrary to [`core::fmt::Write`]), but
/// we cannot use `std::io::Write' as it isn't available in `no_std` (due to
/// `std::io::Error` not being available).
///
/// Also, in our use cases, writes are infallible, so the write function cannot
/// return an `Err`, however it might not be able to write everything, so it
/// returns the number of bytes written.
///
/// See also the tracking issue:
/// <https://github.com/rust-lang/rfcs/issues/2262>.
pub trait IoWrite: core::fmt::Write {
    fn write(&mut self, buf: &[u8]) -> usize;

    fn write_ring_buffer(&mut self, buf: &RingBuffer<'_, u8>) -> usize {
        let (left, right) = buf.as_slices();
        let mut total = 0;
        if let Some(slice) = left {
            total += self.write(slice);
        }
        if let Some(slice) = right {
            total += self.write(slice);
        }
        total
    }
}
