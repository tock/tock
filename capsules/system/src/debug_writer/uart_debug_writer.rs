// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Implementation of [`DebugWriter`] using UART.

use kernel::collections::queue::Queue;
use kernel::collections::ring_buffer::RingBuffer;
use kernel::debug::DebugWriter;
use kernel::hil;
use kernel::utilities::cells::TakeCell;
use kernel::utilities::io_write::IoWrite;
use kernel::ErrorCode;

/// Buffered [`DebugWriter`] implementation using a UART.
///
/// Currently used as a default implementation of DebugWriterComponent.
pub struct UartDebugWriter {
    /// What provides the actual writing mechanism.
    uart: &'static dyn hil::uart::Transmit<'static>,
    /// The buffer that is passed to the writing mechanism.
    output_buffer: TakeCell<'static, [u8]>,
    /// An internal buffer that is used to hold debug!() calls as they come in.
    internal_buffer: TakeCell<'static, RingBuffer<'static, u8>>,
}

impl UartDebugWriter {
    pub fn new(
        uart: &'static dyn hil::uart::Transmit,
        out_buffer: &'static mut [u8],
        internal_buffer: &'static mut RingBuffer<'static, u8>,
    ) -> UartDebugWriter {
        UartDebugWriter {
            uart,
            output_buffer: TakeCell::new(out_buffer),
            internal_buffer: TakeCell::new(internal_buffer),
        }
    }
}

impl DebugWriter for UartDebugWriter {
    fn write(&self, bytes: &[u8], overflow_message: &[u8]) -> usize {
        // If we have a buffer, write to it.
        if let Some(ring_buffer) = self.internal_buffer.take() {
            let available_len = ring_buffer
                .available_len()
                .saturating_sub(overflow_message.len());
            let total_written = if available_len >= bytes.len() {
                for &b in bytes {
                    ring_buffer.enqueue(b);
                }
                bytes.len()
            } else {
                // If we don't have enough space, write as much as we can and
                // then write the overflow message.
                for &b in &bytes[..available_len] {
                    ring_buffer.enqueue(b);
                }
                for &b in overflow_message {
                    ring_buffer.enqueue(b);
                }
                available_len + overflow_message.len()
            };
            // Put the buffer back.
            self.internal_buffer.replace(ring_buffer);
            total_written
        } else {
            // No buffer, so just return the number of bytes.
            bytes.len()
        }
    }

    fn publish(&self) -> usize {
        // Can only publish if we have the output_buffer. If we don't that is
        // fine, we will do it when the transmit done callback happens.
        self.internal_buffer.map_or(0, |ring_buffer| {
            if let Some(out_buffer) = self.output_buffer.take() {
                let mut count = 0;

                for dst in out_buffer.iter_mut() {
                    match ring_buffer.dequeue() {
                        Some(src) => {
                            *dst = src;
                            count += 1;
                        }
                        None => {
                            break;
                        }
                    }
                }

                if count != 0 {
                    // Transmit the data in the output buffer.
                    if let Err((_err, buf)) = self.uart.transmit_buffer(out_buffer, count) {
                        self.output_buffer.put(Some(buf));
                    } else {
                        self.output_buffer.put(None);
                    }
                }
                count
            } else {
                0
            }
        })
    }

    fn flush(&self, writer: &mut dyn IoWrite) {
        self.internal_buffer.map(|ring_buffer| {
            writer.write_ring_buffer(ring_buffer);
        });
    }

    fn available_len(&self) -> usize {
        self.internal_buffer.map_or(0, |rb| rb.available_len())
    }

    fn to_write_len(&self) -> usize {
        self.internal_buffer.map_or(0, |rb| rb.len())
    }
}

impl hil::uart::TransmitClient for UartDebugWriter {
    fn transmitted_buffer(
        &self,
        buffer: &'static mut [u8],
        _tx_len: usize,
        _rcode: Result<(), ErrorCode>,
    ) {
        // Replace this buffer since we are done with it.
        self.output_buffer.replace(buffer);

        if self.internal_buffer.map_or(false, |buf| buf.has_elements()) {
            // Buffer not empty, go around again
            self.publish();
        }
    }
    fn transmitted_word(&self, _rcode: Result<(), ErrorCode>) {}
}

impl core::fmt::Write for UartDebugWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

impl IoWrite for UartDebugWriter {
    fn write(&mut self, bytes: &[u8]) -> usize {
        const FULL_MSG: &[u8] = b"\n*** DEBUG BUFFER FULL ***\n";
        self.internal_buffer.map_or(0, |ring_buffer| {
            let available_len_for_msg = ring_buffer.available_len().saturating_sub(FULL_MSG.len());

            if available_len_for_msg >= bytes.len() {
                for &b in bytes {
                    ring_buffer.enqueue(b);
                }
                bytes.len()
            } else {
                for &b in &bytes[..available_len_for_msg] {
                    ring_buffer.enqueue(b);
                }
                // When the buffer is close to full, print a warning and drop the current
                // string.
                for &b in FULL_MSG {
                    ring_buffer.enqueue(b);
                }
                available_len_for_msg
            }
        })
    }
}
