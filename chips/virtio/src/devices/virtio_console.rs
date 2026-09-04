// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Driver for the VirtIO console device (device type 3), exposed by QEMU (and
//! other hypervisors) as `virtio-serial-device` / `virtio-serial-pci` / etc.
//!
//! This driver implements the UART HIL, letting a virtio console be used
//! anywhere Tock expects a UART.
//!
//! Only the basic (non-multiport) console mode is supported: this driver does
//! not negotiate `VIRTIO_CONSOLE_F_MULTIPORT`, so the device exposes exactly
//! two virtqueues -- a receive queue (0) and a transmit queue (1).
//!
//! Because the device delivers data from the host in host-chosen chunks (there
//! is no way for the guest to request "exactly N bytes"), this driver always
//! keeps a single one-byte buffer posted to the receive queue while a
//! [`hil::uart::Receive::receive_buffer`] call is outstanding, and copies each
//! incoming byte into the client's buffer. This avoids ever having to buffer or
//! discard partial chunks, at the cost of one virtqueue round-trip per received
//! byte.

use core::cell::Cell;

use kernel::ErrorCode;
use kernel::hil;
use kernel::platform::dma_fence::DmaFence;
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::leasable_buffer::{SubSliceMut, SubSliceMutImmut};

use super::super::devices::{VirtIODeviceDriver, VirtIODeviceType};
use super::super::queues::split_queue::{
    SplitVirtqueue, SplitVirtqueueClient, VirtqueueBuffer, VirtqueueReturnBuffer,
};

pub struct VirtIOConsole<'a, F: DmaFence> {
    rxqueue: &'a SplitVirtqueue<'static, 'static, 1, F>,
    txqueue: &'a SplitVirtqueue<'static, 'static, 1, F>,

    tx_client: OptionalCell<&'a dyn hil::uart::TransmitClient>,
    tx_len: Cell<usize>,
    tx_pending: Cell<bool>,

    rx_client: OptionalCell<&'a dyn hil::uart::ReceiveClient>,
    rx_buffer: TakeCell<'static, [u8]>,
    rx_len: Cell<usize>,
    rx_index: Cell<usize>,

    // Single-byte scratch buffer kept posted to the receive queue while a
    // `receive_buffer` call is outstanding.
    rx_chunk: TakeCell<'static, u8>,
}

impl<'a, F: DmaFence> VirtIOConsole<'a, F> {
    pub fn new(
        txqueue: &'a SplitVirtqueue<'static, 'static, 1, F>,
        rxqueue: &'a SplitVirtqueue<'static, 'static, 1, F>,
        rx_chunk: &'static mut u8,
    ) -> VirtIOConsole<'a, F> {
        txqueue.enable_used_callbacks();
        rxqueue.enable_used_callbacks();

        VirtIOConsole {
            rxqueue,
            txqueue,
            tx_client: OptionalCell::empty(),
            tx_len: Cell::new(0),
            tx_pending: Cell::new(false),
            rx_client: OptionalCell::empty(),
            rx_buffer: TakeCell::empty(),
            rx_len: Cell::new(0),
            rx_index: Cell::new(0),
            rx_chunk: TakeCell::new(rx_chunk),
        }
    }

    /// Post the one-byte scratch buffer to the receive queue, if it isn't
    /// already posted (in which case a completion is already in flight and
    /// will re-post it once handled).
    fn post_rx_chunk(&self) {
        if let Some(chunk) = self.rx_chunk.take() {
            let mut chain = [Some(VirtqueueBuffer::DeviceWriteable(SubSliceMut::new(
                core::slice::from_mut(chunk),
            )))];

            if self.rxqueue.provide_buffer_chain(&mut chain).is_err() {
                // Queue is full (should not happen with a single
                // outstanding one-byte chain) -- hand the buffer back so
                // future calls can retry, rather than losing it.
                let VirtqueueBuffer::DeviceWriteable(sub_slice_mut) =
                    chain[0].take().expect("No rx buffer")
                else {
                    panic!("VirtIO console: rx queue returned DeviceReadable buffer")
                };
                let chunk = sub_slice_mut
                    .take()
                    .first_mut()
                    .expect("VirtIO console: rx chunk was resized");
                self.rx_chunk.replace(chunk);
            }
        }
    }

    fn handle_rx_chunk(&self, chunk: &'static mut u8, bytes_used: usize) {
        let byte_received = bytes_used >= 1;
        let byte = *chunk;
        self.rx_chunk.replace(chunk);

        if !byte_received {
            // Spurious/empty completion; keep listening if a receive is
            // still outstanding.
            if self.rx_buffer.is_some() {
                self.post_rx_chunk();
            }
            return;
        }

        let Some(rx_buffer) = self.rx_buffer.take() else {
            // No outstanding receive; drop the byte and go idle.
            return;
        };

        let index = self.rx_index.get();
        // `rx_buffer` was validated to hold at least `rx_len` bytes when the
        // receive was started.
        rx_buffer[index] = byte;
        let new_index = index + 1;

        if new_index >= self.rx_len.get() {
            self.rx_index.set(0);
            let rx_len = self.rx_len.get();
            self.rx_client.map(move |client| {
                client.received_buffer(rx_buffer, rx_len, Ok(()), hil::uart::Error::None)
            });
        } else {
            self.rx_index.set(new_index);
            self.rx_buffer.replace(rx_buffer);
            self.post_rx_chunk();
        }
    }

    fn handle_tx_complete(&self, tx_buffer: &'static mut [u8]) {
        self.tx_pending.set(false);
        let tx_len = self.tx_len.get();
        self.tx_client
            .map(move |client| client.transmitted_buffer(tx_buffer, tx_len, Ok(())));
    }
}

impl<F: DmaFence> SplitVirtqueueClient<'static> for VirtIOConsole<'_, F> {
    fn buffer_chain_ready(
        &self,
        queue_number: u32,
        buffer_chain: &mut [Option<VirtqueueReturnBuffer<'static>>],
        bytes_used: usize,
    ) {
        if queue_number == self.rxqueue.queue_number().unwrap() {
            let VirtqueueBuffer::DeviceWriteable(sub_slice_mut) = buffer_chain[0]
                .take()
                .expect("No rx buffer")
                .virtqueue_buffer
            else {
                panic!("VirtIO console: rx queue returned DeviceReadable buffer")
            };
            let chunk = sub_slice_mut
                .take()
                .first_mut()
                .expect("VirtIO console: rx chunk was resized");
            self.handle_rx_chunk(chunk, bytes_used);
        } else if queue_number == self.txqueue.queue_number().unwrap() {
            let tx = buffer_chain[0].take().expect("No tx buffer");
            let VirtqueueBuffer::DeviceReadable(sub_slice_mut_immut) = tx.virtqueue_buffer else {
                panic!("VirtIO console: tx queue returned DeviceWriteable buffer")
            };
            let SubSliceMutImmut::Mutable(sub_slice_mut) = sub_slice_mut_immut else {
                panic!("VirtIO console: tx buffer SubSliceMutImmut is not mutable")
            };
            self.handle_tx_complete(sub_slice_mut.take());
        } else {
            panic!("VirtIO console: callback from unknown queue");
        }
    }
}

impl<F: DmaFence> VirtIODeviceDriver for VirtIOConsole<'_, F> {
    fn negotiate_features(&self, _offered_features: u64) -> Option<u64> {
        // We only support the basic (non-multiport, non-resizable) console
        // mode, so we don't need any of the offered features.
        Some(0)
    }

    fn device_type(&self) -> VirtIODeviceType {
        VirtIODeviceType::Console
    }
}

impl<F: DmaFence> hil::uart::Configure for VirtIOConsole<'_, F> {
    fn configure(&self, _params: hil::uart::Parameters) -> Result<(), ErrorCode> {
        // A virtio console has no physical line to configure (no baud
        // rate, parity, stop bits, etc.); accept any configuration.
        Ok(())
    }
}

impl<'a, F: DmaFence> hil::uart::Transmit<'a> for VirtIOConsole<'a, F> {
    fn set_transmit_client(&self, client: &'a dyn hil::uart::TransmitClient) {
        self.tx_client.set(client);
    }

    fn transmit_buffer(
        &self,
        tx_buffer: &'static mut [u8],
        tx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if tx_len > tx_buffer.len() {
            return Err((ErrorCode::SIZE, tx_buffer));
        }
        if self.tx_pending.get() {
            return Err((ErrorCode::BUSY, tx_buffer));
        }

        let mut tx_sub_slice = SubSliceMut::new(tx_buffer);
        tx_sub_slice.slice(0..tx_len);

        let mut chain = [Some(VirtqueueBuffer::DeviceReadable(
            SubSliceMutImmut::Mutable(tx_sub_slice),
        ))];

        self.tx_len.set(tx_len);
        self.tx_pending.set(true);

        self.txqueue.provide_buffer_chain(&mut chain).map_err(|e| {
            self.tx_pending.set(false);
            let VirtqueueBuffer::DeviceReadable(SubSliceMutImmut::Mutable(sub_slice_mut)) =
                chain[0].take().expect("No tx buffer")
            else {
                panic!("VirtIO console: tx chain buffer changed type")
            };
            (e, sub_slice_mut.take())
        })
    }

    fn transmit_word(&self, _word: u32) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn transmit_abort(&self) -> Result<(), ErrorCode> {
        // Once submitted to the virtqueue, a transmission cannot be
        // synchronously cancelled.
        Err(ErrorCode::FAIL)
    }
}

impl<'a, F: DmaFence> hil::uart::Receive<'a> for VirtIOConsole<'a, F> {
    fn set_receive_client(&self, client: &'a dyn hil::uart::ReceiveClient) {
        self.rx_client.set(client);
    }

    fn receive_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if rx_len > rx_buffer.len() {
            return Err((ErrorCode::SIZE, rx_buffer));
        }
        if self.rx_buffer.is_some() {
            return Err((ErrorCode::BUSY, rx_buffer));
        }

        self.rx_len.set(rx_len);
        self.rx_index.set(0);
        self.rx_buffer.replace(rx_buffer);
        self.post_rx_chunk();

        Ok(())
    }

    fn receive_word(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn receive_abort(&self) -> Result<(), ErrorCode> {
        // Unsupported: the one-byte scratch buffer may already be posted
        // to the device and cannot be synchronously reclaimed.
        Err(ErrorCode::FAIL)
    }
}
