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

/// Wrapper to convert a binary buffer writer to provide a `core::fmt::Write`
/// interface with offset tracking. This allows a synchronous writer to use
/// an underlying asynchronous write implementation.
///
/// This struct allows a synchronous writer to use the `core::fmt::Write`
/// interface when there is a limited size buffer underneath. This struct tracks
/// where in the overall write has actually been written to the underlying
/// `BinaryWrite` implementation.
///
/// The expected usage of this tool looks like:
///
/// ```ignore
/// let wrapper = WriteToBinaryOffsetWrapper::new(binary_writer);
///
/// // Set the byte index of the long, synchronous write where we should
/// // actually start passing to the binary writer.
/// wrapper.set_offset(offset);
///
/// // Do the long, synchronous write.
/// let _ = wrapper.write_fmt(format_args!(...));
///
/// if wrapper.bytes_remaining() {
///     // Some of the write did not finish (likely that means the binary
///     // writer's buffer filled up).
///     let next_offset = wrapper.get_index();
///
///     // Now wait for the binary write to finish, and start this process
///     // over but from the new offset.
/// } else {
///     // Nothing left to print, we're done!
/// }
/// ```
pub struct WriteToBinaryOffsetWrapper<'a> {
    /// Binary writer implementation that is asynchronous and has a fixed sized
    /// buffer.
    binary_writer: &'a mut dyn BinaryWrite,
    /// Where to start in the long synchronous write.
    offset: usize,
    /// Keep track of where in the long synchronous write we are currently
    /// displaying.
    index: usize,
    /// Track if write() is called, and the `binary_writer` did not print
    /// everything we passed to it. In that case, there are more bytes to write
    /// on the next iteration.
    bytes_remaining: bool,
}

impl<'a> WriteToBinaryOffsetWrapper<'a> {
    pub fn new(binary_writer: &'a mut dyn BinaryWrite) -> Self {
        Self {
            binary_writer,
            index: 0,
            offset: 0,
            bytes_remaining: false,
        }
    }

    /// Set the byte to start printing from on this iteration. Call this before
    /// calling `Write`.
    pub fn set_offset(&mut self, offset: usize) {
        self.offset = offset;
    }

    /// After printing, get the index we left off on to use as the offset for
    /// the next iteration.
    pub fn get_index(&self) -> usize {
        self.index
    }

    /// After printing, check if there is more to print that the binary_writer
    /// did not print.
    pub fn bytes_remaining(&self) -> bool {
        self.bytes_remaining
    }
}

impl core::fmt::Write for WriteToBinaryOffsetWrapper<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let string_len = s.len();
        if self.index + string_len < self.offset {
            // We are still waiting for `self.offset` bytes to be send before we
            // actually start printing.
            self.index += string_len;
            Ok(())
        } else {
            // We need to be printing at least some of this.
            let start = if self.offset <= self.index {
                // We're past our offset, so we can display this entire str.
                0
            } else {
                // We want to start in the middle.
                self.offset.saturating_sub(self.index)
            };

            // Calculate the number of bytes we are going to pass to the
            // binary_writer.
            let to_send = string_len - start;

            // Actually do the write. This will return how many bytes it was
            // able to print.
            let ret = self
                .binary_writer
                .write_buffer(&(s).as_bytes()[start..string_len]);

            match ret {
                Ok(bytes_sent) => {
                    // Update our index based on how much was sent and how much
                    // (if any) we skipped over.
                    self.index += bytes_sent + start;

                    // Check if less was sent than we asked. This signals that
                    // we will have more work to do on the next iteration.
                    if to_send > bytes_sent {
                        self.bytes_remaining = true;
                    }

                    Ok(())
                }
                Err(()) => Err(core::fmt::Error),
            }
        }
    }
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
