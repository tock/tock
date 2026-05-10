// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Basic binary write interface and supporting structs.
//!
//! This simple trait provides a synchronous interface for printing an arbitrary
//! binary slice. This mirrors the `core::fmt::Write` interface but doesn't
//! expect a `&str`.

/// Interface for writing an arbitrary buffer.
pub trait BinaryWrite {
    /// Write the `buffer` to some underlying print mechanism.
    ///
    /// Returns `Ok(usize)` on success with the number of bytes from `buffer`
    /// that were written. Returns `Err(())` on any error.
    fn write_buffer(&mut self, buffer: &[u8]) -> Result<usize, ()>;
}

/// Provide a `BinaryWrite` interface on top of a synchronous `core::fmt::Write`
/// interface.
///
/// Note, this MUST only be used to reverse the output of
/// `WriteToBinaryOffsetWrapper`. That is, this assume that the binary strings
/// are valid UTF-8, which will be the case if the binary buffer comes from some
/// `core::fmt::Write` operation originally.
pub(crate) struct BinaryToWriteWrapper<'a> {
    writer: &'a mut dyn core::fmt::Write,
}

impl<'a> BinaryToWriteWrapper<'a> {
    pub(crate) fn new(writer: &'a mut dyn core::fmt::Write) -> Self {
        Self { writer }
    }
}

impl BinaryWrite for BinaryToWriteWrapper<'_> {
    fn write_buffer(&mut self, buffer: &[u8]) -> Result<usize, ()> {
        // Convert the binary string to UTF-8 so we can print it as a string. If
        // this is not actually a UTF-8 string, then return Err(()).
        let s = core::str::from_utf8(buffer).or(Err(()))?;
        let _ = self.writer.write_str(s);
        Ok(buffer.len())
    }
}
