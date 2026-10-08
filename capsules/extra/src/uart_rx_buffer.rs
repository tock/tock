// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! UART receive buffer layer to ensure kernel is always ready to receive.
//!
//! This can be used on top of a chip's UART receive implementation to ensure
//! that there is always a buffer ready to receive a byte from the hardware.
//! Then, any UART receives that come from above (e.g., from userspace) will
//! either be immediately handled from data already stored in this layer, or
//! wait until enough data has been read from below.
//!
//! This module only implements `hil::uart::Receive`. To use it as a drop-in
//! replacement for a full `hil::uart::Uart`, pair it with [`UartRxBufferFull`],
//! which forwards `Transmit`/`Configure` straight to an underlying
//! `hil::uart::Uart`.
//!
//! # Usage Stack
//!
//! ```text
//!   ┌────────────────────────────────────────────────────┐
//!   │                                                    │
//!   │    UART stack                                      │
//!   │                                                    │
//!   └────────────────────────────────────────────────────┘
//!     hil::Uart
//!   ┌────────────────────────────────────────────────────┐
//!   │                                                    │
//!   │     UartRxBufferFull                               │
//!   │     (provided in this module)                      │
//!   │                                                    │
//!   └────────────────────────────────────────────────────┘
//!      hil::uart::Transmit         hil::Uart::Receive
//!      hil::uart::Configure
//!          │                     ┌───────────────────────┐
//!          │                     │                       │
//!          │                     │     UartRxBuffer      │
//!          │                     │     (this module)     │
//!          │                     │                       │
//!          │                     └───────────────────────┘
//!          ↓                                   ↓
//!                         hil::Uart
//!   ┌────────────────────────────────────────────────────┐
//!   │                                                    │
//!   │  Chip Uart HW                                      │
//!   │                                                    │
//!   └────────────────────────────────────────────────────┘
//! ```

use core::cmp;

use kernel::ErrorCode;
use kernel::collections::queue::Queue;
use kernel::collections::ring_buffer::RingBuffer;
use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil::uart;
use kernel::utilities::cells::{MapCell, OptionalCell, TakeCell};

/// Default size, in bytes, of the ring buffer used to hold
/// received-but-undelivered data. Boards can size their ring buffer
/// differently; see [`UartRxBuffer::new`].
pub const DEFAULT_RING_BUFFER_SIZE_BYTE: usize = 512;

/// Wraps a `hil::uart::Uart` and a [`UartRxBuffer`] on top of it to present a
/// full `hil::uart::Uart` interface.
pub struct UartRxBufferFull<'a, U: uart::Uart<'a>, URX: uart::Receive<'a>> {
    uart: &'a U,
    uart_rx: &'a UartRxBuffer<'a, URX>,
}

impl<'a, U: uart::Uart<'a>, URX: uart::Receive<'a>> UartRxBufferFull<'a, U, URX> {
    pub fn new(uart: &'a U, uart_rx: &'a UartRxBuffer<'a, URX>) -> Self {
        Self { uart, uart_rx }
    }
}

impl<'a, U: uart::Uart<'a>, URX: uart::Receive<'a>> uart::Configure
    for UartRxBufferFull<'a, U, URX>
{
    fn configure(&self, params: uart::Parameters) -> Result<(), ErrorCode> {
        self.uart.configure(params)
    }
}

impl<'a, U: uart::Uart<'a>, URX: uart::Receive<'a>> uart::Transmit<'a>
    for UartRxBufferFull<'a, U, URX>
{
    fn set_transmit_client(&self, client: &'a dyn uart::TransmitClient) {
        self.uart.set_transmit_client(client);
    }

    fn transmit_buffer(
        &self,
        tx_buffer: &'static mut [u8],
        tx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        self.uart.transmit_buffer(tx_buffer, tx_len)
    }

    fn transmit_word(&self, word: u32) -> Result<(), ErrorCode> {
        self.uart.transmit_word(word)
    }

    fn transmit_abort(&self) -> Result<(), ErrorCode> {
        self.uart.transmit_abort()
    }
}

impl<'a, U: uart::Uart<'a>, URX: uart::Receive<'a>> uart::Receive<'a>
    for UartRxBufferFull<'a, U, URX>
{
    fn set_receive_client(&self, client: &'a dyn uart::ReceiveClient) {
        self.uart_rx.set_receive_client(client);
    }

    fn receive_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        self.uart_rx.receive_buffer(rx_buffer, rx_len)
    }

    fn receive_word(&self) -> Result<(), ErrorCode> {
        self.uart_rx.receive_word()
    }

    fn receive_abort(&self) -> Result<(), ErrorCode> {
        self.uart_rx.receive_abort()
    }
}

/// A client [`UartRxBuffer::receive_buffer`] call that needs to wait on more
/// RX data.
struct PendingRequest {
    /// Buffer from the upper layer to fill.
    buffer: &'static mut [u8],
    /// Total number of RX bytes requested.
    rx_len: usize,
    /// How many bytes have been received and copied in.
    filled: usize,
}

/// A client [`UartRxBuffer::receive_buffer`] where we already have the data,
/// and just need to wait for the deferred call.
struct ReadyRequest {
    buffer: &'static mut [u8],
    filled: usize,
    rval: Result<(), ErrorCode>,
    error: uart::Error,
}

/// Buffers UART receive data ahead of a client -- see the module docs.
pub struct UartRxBuffer<'a, U: uart::Receive<'a>> {
    uart: &'a U,
    deferred_call: DeferredCall,
    client: OptionalCell<&'a dyn uart::ReceiveClient>,

    /// One-byte buffer continuously handed to `uart.receive_buffer()` so
    /// there's always an outstanding receive to catch the next byte.
    rx_buffer: TakeCell<'static, [u8]>,

    /// Ring buffer of bytes received but not yet delivered to a client.
    ring: TakeCell<'static, RingBuffer<'static, u8>>,

    /// Set if there is an RX request that we don't have enough buffered data to
    /// handle yet.
    pending: MapCell<PendingRequest>,

    /// Set if we can immediately handle an RX request but need to wait for a
    /// deferred call to trigger the callback.
    ready: MapCell<ReadyRequest>,
}

impl<'a, U: uart::Receive<'a>> UartRxBuffer<'a, U> {
    pub fn new(
        uart: &'a U,
        rx_byte_buffer: &'static mut [u8],
        ring_buffer: &'static mut RingBuffer<'static, u8>,
    ) -> Self {
        Self {
            uart,
            deferred_call: DeferredCall::new(),
            client: OptionalCell::empty(),
            rx_buffer: TakeCell::new(rx_byte_buffer),
            ring: TakeCell::new(ring_buffer),
            pending: MapCell::empty(),
            ready: MapCell::empty(),
        }
    }

    /// Starts the background collection by issuing the first single-byte
    /// receive.
    pub fn start_receiving(&'static self) {
        self.rx_buffer.take().map(|byte_buffer| {
            if let Err((_ecode, buffer)) = self.uart.receive_buffer(byte_buffer, 1) {
                self.rx_buffer.replace(buffer);
            }
        });
    }

    /// Moves the oldest `n` buffered bytes into `dest[..n]`. `n` must be
    /// `<= self.buffered_len()`.
    fn drain_into(&self, dest: &mut [u8], n: usize) {
        self.ring.map(|ring| {
            for dest_byte in dest[..n].iter_mut() {
                if let Some(byte) = ring.dequeue() {
                    *dest_byte = byte;
                }
            }
        });
    }

    /// Number of bytes currently buffered in the ring buffer.
    fn buffered_len(&self) -> usize {
        self.ring.map_or(0, |ring| ring.len())
    }
}

impl<'a, U: uart::Receive<'a>> uart::Receive<'a> for UartRxBuffer<'a, U> {
    fn set_receive_client(&self, client: &'a dyn uart::ReceiveClient) {
        self.client.set(client);
    }

    fn receive_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if rx_len == 0 || rx_len > rx_buffer.len() {
            return Err((ErrorCode::SIZE, rx_buffer));
        }
        if self.pending.is_some() || self.ready.is_some() {
            return Err((ErrorCode::BUSY, rx_buffer));
        }

        // Drain whatever is already buffered right away, regardless of
        // whether it's enough to satisfy the whole request, so the ring
        // buffer doesn't sit full (or get closer to full) while we wait
        // for the rest.
        let available = self.buffered_len();
        let n = cmp::min(available, rx_len);
        self.drain_into(rx_buffer, n);

        if n == rx_len {
            // Fully satisfied already: complete on the next deferred call
            // tick so the callback is always asynchronous.
            self.ready.replace(ReadyRequest {
                buffer: rx_buffer,
                filled: n,
                rval: Ok(()),
                error: uart::Error::None,
            });
            self.deferred_call.set();
        } else {
            // Not enough buffered yet: wait for the remaining bytes to
            // arrive from the UART below -- see `received_buffer()`. No
            // short-read callback in the meantime.
            self.pending.replace(PendingRequest {
                buffer: rx_buffer,
                filled: n,
                rx_len,
            });
        }
        Ok(())
    }

    fn receive_word(&self) -> Result<(), ErrorCode> {
        // Unsupported: the background collection in `received_buffer()`
        // already owns the UART below's receive channel exclusively.
        Err(ErrorCode::FAIL)
    }

    fn receive_abort(&self) -> Result<(), ErrorCode> {
        if let Some(mut ready) = self.ready.take() {
            ready.rval = Err(ErrorCode::CANCEL);
            ready.error = uart::Error::Aborted;
            self.ready.replace(ready);
            // The deferred call from the `receive_buffer()` that created
            // this is already pending; nothing more to schedule.
            return Err(ErrorCode::BUSY);
        }
        if let Some(pending) = self.pending.take() {
            // Abort means what was pending is now "ready" and we can pass it
            // back after the deferred call triggers.
            self.ready.replace(ReadyRequest {
                buffer: pending.buffer,
                filled: pending.filled,
                rval: Err(ErrorCode::CANCEL),
                error: uart::Error::Aborted,
            });
            self.deferred_call.set();
            return Err(ErrorCode::BUSY);
        }
        Ok(())
    }
}

impl<'a, U: uart::Receive<'a>> uart::ReceiveClient for UartRxBuffer<'a, U> {
    fn received_buffer(
        &self,
        buffer: &'static mut [u8],
        rx_len: usize,
        rcode: Result<(), ErrorCode>,
        _error: uart::Error,
    ) {
        // Add the RXed byte to our ring buffer.
        if rcode.is_ok() && rx_len >= 1 {
            self.ring.map(|ring| {
                ring.enqueue(buffer[0]);
            });
        }

        // Keep the background collection going.
        if let Err((_ecode, buffer)) = self.uart.receive_buffer(buffer, 1) {
            self.rx_buffer.replace(buffer);
        }

        // If a client is waiting on more data, drain whatever is newly
        // available into its buffer right away (rather than leaving it in
        // the ring), and complete the request once it's full.
        if let Some(mut req) = self.pending.take() {
            let available = self.buffered_len();
            let needed = req.rx_len - req.filled;
            let n = cmp::min(available, needed);
            self.drain_into(&mut req.buffer[req.filled..], n);
            req.filled += n;

            if req.filled >= req.rx_len {
                let PendingRequest { buffer, rx_len, .. } = req;
                self.client.map(|client| {
                    client.received_buffer(buffer, rx_len, Ok(()), uart::Error::None)
                });
            } else {
                self.pending.replace(req);
            }
        }
    }
}

impl<'a, U: uart::Receive<'a>> DeferredCallClient for UartRxBuffer<'a, U> {
    fn handle_deferred_call(&self) {
        if let Some(ReadyRequest {
            buffer,
            filled,
            rval,
            error,
        }) = self.ready.take()
        {
            self.client
                .map(|client| client.received_buffer(buffer, filled, rval, error));
        }
    }

    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}
