// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Buffers UART receive data ahead of a client, decoupling when bytes
//! arrive from when the client asks for them.
//!
//! This sits between a UART (`hil::uart::Uart`) and whatever above it
//! wants to `hil::uart::Receive` from it -- typically
//! `capsules_core::virtualizers::virtual_uart::MuxUart`, in place of the
//! raw UART peripheral. It passes `Configure` and `Transmit` straight
//! through; only `Receive` is intercepted.
//!
//! In the background, it continuously receives one byte at a time from the
//! UART below and appends each to an internal ring buffer, independent of
//! whether the client above has an outstanding [`hil::uart::Receive::receive_buffer`]
//! call. When the client does call [`hil::uart::Receive::receive_buffer`]:
//! - If the ring buffer already holds any bytes, they're returned right
//!   away (a short read, if fewer than `rx_len` are buffered).
//! - If the ring buffer is empty, the call instead waits for `rx_len` bytes
//!   to accumulate in the ring buffer before returning (a full read).
//!
//! Usage
//! -----
//! ```rust,ignore
//! # use kernel::static_init;
//! static mut RX_BYTE: [u8; 1] = [0; 1];
//! static mut RX_RING: [u8; 512] = [0; 512];
//! let uart_rx_buffer = static_init!(
//!     capsules_extra::uart_rx_buffer::UartRxBuffer<'static>,
//!     capsules_extra::uart_rx_buffer::UartRxBuffer::new(
//!         &peripherals.uart1,
//!         &mut RX_BYTE,
//!         &mut RX_RING,
//!     )
//! );
//! uart_rx_buffer.register();
//! uart_rx_buffer.start_receiving();
//! // Then build a `MuxUart`/`Console`/etc. on `uart_rx_buffer` as if it
//! // were the UART itself.
//! ```

use core::cell::Cell;
use core::cmp;

use kernel::ErrorCode;
use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil::uart;
use kernel::utilities::cells::{MapCell, OptionalCell, TakeCell};

/// A client [`UartRxBuffer::receive_buffer`] call that hasn't been
/// satisfied yet: the ring buffer was empty when it was made, so it's
/// waiting for `rx_len` bytes to accumulate.
struct PendingRequest {
    buffer: &'static mut [u8],
    rx_len: usize,
}

/// A client [`UartRxBuffer::receive_buffer`] call whose result is already
/// known, queued for a deferred call so the completion callback is always
/// asynchronous (never made from within the call that started it).
struct ReadyRequest {
    buffer: &'static mut [u8],
    filled: usize,
    rval: Result<(), ErrorCode>,
    error: uart::Error,
}

/// Buffers UART receive data ahead of a client -- see the module docs.
pub struct UartRxBuffer<'a> {
    /// The UART underneath: `Transmit`/`Configure` calls pass straight
    /// through to it; its `Receive` side is exclusively owned by this
    /// capsule's own background collection (see `received_buffer` below).
    uart: &'a dyn uart::Uart<'a>,
    /// This capsule's client -- whatever is above it, e.g. a `MuxUart`.
    client: OptionalCell<&'a dyn uart::ReceiveClient>,

    /// One-byte buffer continuously handed to `uart.receive_buffer()` so
    /// there's always an outstanding receive to catch the next byte.
    rx_byte: TakeCell<'static, [u8]>,

    /// Ring buffer of bytes received but not yet delivered to a client.
    /// `head` is the index of the oldest buffered byte; `count` bytes
    /// starting there (wrapping) are valid.
    ring: TakeCell<'static, [u8]>,
    head: Cell<usize>,
    count: Cell<usize>,

    pending: MapCell<PendingRequest>,
    ready: MapCell<ReadyRequest>,
    deferred_call: DeferredCall,
}

impl<'a> UartRxBuffer<'a> {
    /// `rx_byte_buffer` only ever holds one byte at a time; it just needs
    /// to be at least one byte long. `ring_buffer` is the backing store
    /// for buffered-but-undelivered bytes -- the caller should size it
    /// (at least 512 bytes) for how far ahead of the client it wants
    /// reception to be able to run.
    pub fn new(
        uart: &'a dyn uart::Uart<'a>,
        rx_byte_buffer: &'static mut [u8],
        ring_buffer: &'static mut [u8],
    ) -> Self {
        Self {
            uart,
            client: OptionalCell::empty(),
            rx_byte: TakeCell::new(rx_byte_buffer),
            ring: TakeCell::new(ring_buffer),
            head: Cell::new(0),
            count: Cell::new(0),
            pending: MapCell::empty(),
            ready: MapCell::empty(),
            deferred_call: DeferredCall::new(),
        }
    }

    /// Registers this capsule as `uart`'s receive client and starts the
    /// background collection. Must be called once, after this capsule has
    /// been placed at `'static` and [`Self::register`]ed, before any
    /// client calls [`Self::receive_buffer`].
    pub fn start_receiving(&'static self) {
        self.uart.set_receive_client(self);
        self.rx_byte.take().map(|byte_buffer| {
            if let Err((_ecode, buffer)) = self.uart.receive_buffer(byte_buffer, 1) {
                self.rx_byte.replace(buffer);
            }
        });
    }

    /// Appends `byte` to the ring buffer, silently dropping it if the ring
    /// buffer is full: a client that isn't reading fast enough loses new
    /// data rather than this capsule corrupting data a client hasn't read
    /// yet.
    fn push_byte(&self, byte: u8) {
        let len = self.ring.map_or(0, |ring| ring.len());
        let count = self.count.get();
        if len == 0 || count >= len {
            return;
        }
        let tail = (self.head.get() + count) % len;
        self.ring.map(|ring| ring[tail] = byte);
        self.count.set(count + 1);
    }

    /// Moves the oldest `n` buffered bytes into `dest[..n]`. `n` must be
    /// `<= self.count.get()`.
    fn drain_into(&self, dest: &mut [u8], n: usize) {
        let len = self.ring.map_or(1, |ring| ring.len());
        let head = self.head.get();
        self.ring.map(|ring| {
            for (i, dest_byte) in dest[..n].iter_mut().enumerate() {
                *dest_byte = ring[(head + i) % len];
            }
        });
        self.head.set((head + n) % len);
        self.count.set(self.count.get() - n);
    }
}

impl<'a> uart::Configure for UartRxBuffer<'a> {
    fn configure(&self, params: uart::Parameters) -> Result<(), ErrorCode> {
        self.uart.configure(params)
    }
}

impl<'a> uart::Transmit<'a> for UartRxBuffer<'a> {
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

impl<'a> uart::Receive<'a> for UartRxBuffer<'a> {
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

        let available = self.count.get();
        if available > 0 {
            // Something is already buffered: hand back up to `rx_len`
            // bytes now (a short read if fewer than `rx_len` are
            // available), completing on the next deferred call tick so
            // the callback is always asynchronous.
            let n = cmp::min(available, rx_len);
            self.drain_into(rx_buffer, n);
            let rval = if n == rx_len {
                Ok(())
            } else {
                Err(ErrorCode::SIZE)
            };
            self.ready.replace(ReadyRequest {
                buffer: rx_buffer,
                filled: n,
                rval,
                error: uart::Error::None,
            });
            self.deferred_call.set();
        } else {
            // Nothing buffered yet: wait for `rx_len` bytes to arrive from
            // the UART below -- see `received_buffer()`.
            self.pending.replace(PendingRequest {
                buffer: rx_buffer,
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
            self.ready.replace(ReadyRequest {
                buffer: pending.buffer,
                filled: 0,
                rval: Err(ErrorCode::CANCEL),
                error: uart::Error::Aborted,
            });
            self.deferred_call.set();
            return Err(ErrorCode::BUSY);
        }
        Ok(())
    }
}

impl<'a> uart::ReceiveClient for UartRxBuffer<'a> {
    fn received_buffer(
        &self,
        buffer: &'static mut [u8],
        rx_len: usize,
        rcode: Result<(), ErrorCode>,
        _error: uart::Error,
    ) {
        // `buffer` is `rx_byte`: one byte, back from the UART below.
        if rcode.is_ok() && rx_len >= 1 {
            self.push_byte(buffer[0]);
        }

        // Keep the background collection going regardless of whether that
        // byte was usable, so one dropped/errored byte doesn't stall
        // reception.
        if let Err((_ecode, buffer)) = self.uart.receive_buffer(buffer, 1) {
            self.rx_byte.replace(buffer);
        }

        // If a client is waiting for the ring buffer to fill up, see if it
        // has now.
        if let Some(req) = self.pending.take() {
            if self.count.get() >= req.rx_len {
                let PendingRequest { buffer, rx_len } = req;
                self.drain_into(buffer, rx_len);

                self.client.map(|client| {
                    client.received_buffer(buffer, rx_len, Ok(()), uart::Error::None)
                });
            } else {
                self.pending.replace(req);
            }
        }
    }
}

impl<'a> DeferredCallClient for UartRxBuffer<'a> {
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
