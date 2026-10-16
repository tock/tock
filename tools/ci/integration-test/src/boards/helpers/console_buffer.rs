// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

/// Number of trailing bytes of pending output to include in a timeout error.
const TIMEOUT_CONTEXT_BYTES: usize = 200;

/// Buffer for output received from a board's console.
///
/// This tracks the output which has not yet been consumed by
/// [`ConsoleBuffer::take_through`], as well as all output received so far.
#[derive(Default)]
pub struct ConsoleBuffer {
    pending: Vec<u8>,
    received: Vec<u8>,
}

impl ConsoleBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Append `bytes` received from the console.
    pub fn push(&mut self, bytes: &[u8]) {
        self.pending.extend_from_slice(bytes);
        self.received.extend_from_slice(bytes);
    }

    /// Remove and return the pending output up to and including the first
    /// `needle`. This returns `None` if the pending output does not contain
    /// `needle`.
    pub fn take_through(&mut self, needle: &str) -> Option<String> {
        let end = self
            .pending
            .windows(needle.len())
            .position(|w| w == needle.as_bytes())?
            + needle.len();
        let taken: Vec<u8> = self.pending.drain(..end).collect();
        Some(String::from_utf8_lossy(&taken).into_owned())
    }

    /// Construct the error for a timeout while waiting for `needle`. The error
    /// includes the last [`TIMEOUT_CONTEXT_BYTES`] bytes of pending output.
    pub fn timeout_error(&self, needle: &str) -> anyhow::Error {
        let tail = &self.pending[self.pending.len().saturating_sub(TIMEOUT_CONTEXT_BYTES)..];
        anyhow::anyhow!(
            "timed out waiting for {needle:?}, last output: {:?}",
            String::from_utf8_lossy(tail)
        )
    }

    /// Return all output received so far.
    pub fn transcript(&self) -> &[u8] {
        &self.received
    }
}

#[cfg(test)]
mod tests {
    use super::ConsoleBuffer;

    #[test]
    fn take_through_consumes_pending_output() {
        let mut console = ConsoleBuffer::new();
        console.push(b"ab");
        assert_eq!(console.take_through("c"), None);
        console.push(b"cdc");
        assert_eq!(console.take_through("c").as_deref(), Some("abc"));
        assert_eq!(console.take_through("c").as_deref(), Some("dc"));
        assert_eq!(console.take_through("c"), None);
        assert_eq!(console.transcript(), b"abcdc");
    }
}
