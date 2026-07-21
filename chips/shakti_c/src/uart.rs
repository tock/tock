// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! UART driver for the SHAKTI C-Class test SoC.
//!
//! The SHAKTI test-SoC UART (`mkuart_axi4`, MMIO base `0x0001_1300`) has no
//! interrupt line wired in the simulation SoC (the PLIC `meip`/`seip` inputs are
//! tied to zero), so this driver is fully **polled**. Transmit completion is
//! signalled to the client from a [`DeferredCall`].
//!
//! The register layout and the transmit handshake are taken from the proven
//! the bare-metal bring-up path: poll the status
//! register at offset `0xC` and wait while `tx_full` (bit 1) is set, then write
//! the byte to the TX register at offset `0x4`.

use core::cell::Cell;

use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil;
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

pub const UART0_BASE: StaticRef<ShaktiUartRegisters> =
    unsafe { StaticRef::new(0x0001_1300 as *const ShaktiUartRegisters) };

register_structs! {
    pub ShaktiUartRegisters {
        /// Baud divisor.
        (0x00 => baud: ReadWrite<u16>),
        (0x02 => _reserved0),
        /// Transmit byte: writing a byte here enqueues it for transmission.
        (0x04 => tx: ReadWrite<u8>),
        (0x05 => _reserved1),
        /// Receive byte.
        (0x08 => rx: ReadOnly<u8>),
        (0x09 => _reserved2),
        /// Status register (see [`status`]).
        (0x0C => status: ReadOnly<u16, status::Register>),
        (0x0E => _reserved3),
        (0x10 => @END),
    }
}

register_bitfields![u16,
    status [
        /// Set when the transmitter has fully drained (no byte in flight).
        /// Validated against the the bare-metal bring-up `exit()` drain.
        transmission_done OFFSET(0) NUMBITS(1) [],
        /// TX FIFO full: while set, the TX register cannot accept a new byte.
        /// Validated against the the bare-metal bring-up hello.
        tx_full OFFSET(1) NUMBITS(1) [],
        /// RX data available. NOTE: tentative — the SoC UART's exact RX status
        /// bit was not exercised by the bare-metal bring-up; validate before relying on RX.
        rx_not_empty OFFSET(3) NUMBITS(1) [],
    ],
];

pub struct Uart<'a> {
    registers: StaticRef<ShaktiUartRegisters>,
    tx_client: OptionalCell<&'a dyn hil::uart::TransmitClient>,
    rx_client: OptionalCell<&'a dyn hil::uart::ReceiveClient>,
    tx_buffer: TakeCell<'static, [u8]>,
    tx_len: Cell<usize>,
    rx_buffer: TakeCell<'static, [u8]>,
    rx_len: Cell<usize>,
    deferred_call: DeferredCall,
}

impl<'a> Uart<'a> {
    pub fn new(base: StaticRef<ShaktiUartRegisters>) -> Uart<'a> {
        Uart {
            registers: base,
            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),
            tx_buffer: TakeCell::empty(),
            tx_len: Cell::new(0),
            rx_buffer: TakeCell::empty(),
            rx_len: Cell::new(0),
            deferred_call: DeferredCall::new(),
        }
    }

    /// Blocking single-byte transmit using the proven status handshake.
    fn send_byte(&self, byte: u8) {
        while self.registers.status.is_set(status::tx_full) {}
        self.registers.tx.set(byte);
    }

    /// Blocking transmit of a slice. Used for panic / synchronous debug output.
    pub fn transmit_sync(&self, bytes: &[u8]) {
        for &b in bytes {
            self.send_byte(b);
        }
        // Wait for the last byte to fully serialize (status.transmission_done,
        // bit 0) so callers that immediately stop the machine don't truncate it.
        while !self.registers.status.is_set(status::transmission_done) {}
    }
}

impl DeferredCallClient for Uart<'_> {
    fn register(&'static self) {
        self.deferred_call.register(self);
    }

    fn handle_deferred_call(&self) {
        // The whole TX buffer is written synchronously in `transmit_buffer`, so
        // here we simply return it to the client, off the original call stack.
        if let Some(buffer) = self.tx_buffer.take() {
            let len = self.tx_len.get();
            self.tx_client.map(|client| {
                client.transmitted_buffer(buffer, len, Ok(()));
            });
        }
    }
}

impl hil::uart::Configure for Uart<'_> {
    fn configure(&self, _params: hil::uart::Parameters) -> Result<(), ErrorCode> {
        // The Verilator testbench captures characters regardless of the divisor,
        // so configuration is a no-op in simulation. The reset divisor is left
        // in place. (A real board would program `baud` from the core clock.)
        Ok(())
    }
}

impl<'a> hil::uart::Transmit<'a> for Uart<'a> {
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
        if self.tx_buffer.is_some() {
            return Err((ErrorCode::BUSY, tx_buffer));
        }

        // Polled UART: write the whole buffer synchronously, then notify the
        // client from a deferred call (no TX interrupt is available).
        for &b in tx_buffer[..tx_len].iter() {
            self.send_byte(b);
        }
        self.tx_len.set(tx_len);
        self.tx_buffer.replace(tx_buffer);
        self.deferred_call.set();
        Ok(())
    }

    fn transmit_abort(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn transmit_word(&self, _word: u32) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }
}

impl<'a> hil::uart::Receive<'a> for Uart<'a> {
    fn set_receive_client(&self, client: &'a dyn hil::uart::ReceiveClient) {
        self.rx_client.set(client);
    }

    fn receive_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        // RX is not yet validated on this SoC UART (see module docs / the
        // `rx_not_empty` status bit note). For now accept and hold the buffer;
        // a later revision will poll the RX path once it is exercised in sim.
        if rx_len > rx_buffer.len() {
            return Err((ErrorCode::SIZE, rx_buffer));
        }
        self.rx_len.set(rx_len);
        self.rx_buffer.replace(rx_buffer);
        Ok(())
    }

    fn receive_abort(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn receive_word(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }
}
