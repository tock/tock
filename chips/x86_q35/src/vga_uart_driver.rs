// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! VgaUart` — a **synchronous, write-only** façade that lets capsules
//! use a `hil::uart::Uart`-style interface while we actually write to
//! the VGA text buffer, not a serial port.
//!
//! ## Key design points
//!
//! - Implements the *minimum subset* of `Transmit` required by `MuxUart`.
//!   All writes copy bytes to the global `VGA_TEXT` then schedule a deferred
//!   call to invoke the transmit callback (split-phase contract).
//! - **Receive / abort / re-configure** operations just return
//!   `ErrorCode::NOSUPPORT` — VGA is output-only.

use crate::vga::Vga;
use core::mem::MaybeUninit;
use core::{cell::Cell, cmp};
use kernel::component::Component;
use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil::uart::{Configure, Parameters, Receive, ReceiveClient, Transmit, TransmitClient};
use kernel::utilities::cells::TakeCell;
use kernel::ErrorCode;
use tock_cells::optional_cell::OptionalCell;

/// UART-compatible wrapper around the VGA text writer.
pub struct VgaText<'a> {
    vga_buffer: Vga,
    tx_client: OptionalCell<&'a dyn TransmitClient>,
    rx_client: OptionalCell<&'a dyn ReceiveClient>,
    deferred_call: DeferredCall,
    pending_buf: TakeCell<'static, [u8]>,
    pending_len: Cell<usize>,
}

impl VgaText<'_> {
    pub fn new() -> Self {
        Self {
            vga_buffer: Vga::new(),
            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),
            deferred_call: DeferredCall::new(),
            pending_buf: TakeCell::empty(),
            pending_len: Cell::new(0),
        }
    }

    fn fire_tx_callback(&self, buf: &'static mut [u8], len: usize) {
        self.tx_client.map(|client| {
            client.transmitted_buffer(buf, len, Ok(()));
        });
    }
}

// DeferredCallClient implementation
impl DeferredCallClient for VgaText<'_> {
    fn handle_deferred_call(&self) {
        if let Some(buf) = self.pending_buf.take() {
            let len = self.pending_len.get();
            self.fire_tx_callback(buf, len);
        }
    }

    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}

// Transmit for Vga
impl<'a> Transmit<'a> for VgaText<'a> {
    fn set_transmit_client(&self, client: &'a dyn TransmitClient) {
        self.tx_client.set(client);
    }

    fn transmit_buffer(
        &self,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        let write_len = cmp::min(len, buffer.len());
        for &byte in &buffer[..write_len] {
            self.vga_buffer.write_byte(byte);
        }
        self.pending_buf.replace(buffer);
        self.pending_len.set(len);
        self.deferred_call.set();
        Ok(())
    }

    fn transmit_word(&self, _word: u32) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }

    fn transmit_abort(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }
}

// Receive for Vga
impl<'a> Receive<'a> for VgaText<'a> {
    fn set_receive_client(&self, client: &'a dyn ReceiveClient) {
        self.rx_client.set(client);
    }

    fn receive_buffer(
        &self,
        buffer: &'static mut [u8],
        _len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        Err((ErrorCode::NOSUPPORT, buffer))
    }

    fn receive_word(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }

    fn receive_abort(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }
}

// Configure for Vga
impl Configure for VgaText<'_> {
    fn configure(&self, _params: Parameters) -> Result<(), ErrorCode> {
        Ok(())
    }
}
// VgaText component: builds a single global `VgaText` and wires it up to DeferredCall.
//
// This follows the Tock Component pattern so boards can do:
//   let vga = VgaTextComponent::new().finalize(vga_text_component_static!());
//

/// Zero-sized builder for `VgaText`.
pub struct VgaTextComponent;

impl VgaTextComponent {
    /// Constructor; zero-sized
    pub const fn new() -> Self {
        Self
    }
}

impl Component for VgaTextComponent {
    /// The board provides a static buffer where we will place the `VgaText`.
    type StaticInput = &'static mut MaybeUninit<VgaText<'static>>;
    /// We return a `'static` reference so the board can store it long-term.
    type Output = &'static VgaText<'static>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        // Place-construct `VgaText` in the caller-provided static buffer.
        let v = s.write(VgaText::new());

        // Register with the kernel’s DeferredCall so `VgaText` can deliver
        // its split-phase transmit completion callbacks later.
        kernel::deferred_call::DeferredCallClient::register(v);

        // Hand back a stable reference to the initialized instance.
        v
    }
}

/// Macro to allocate the static storage required by the component.
///
/// Usage from chip:
///   let vga = VgaTextComponent::new().finalize(vga_text_component_static!());
#[macro_export]
macro_rules! vga_text_component_static {
    () => {{
        kernel::static_buf!($crate::vga_uart_driver::VgaText<'static>)
    }};
}
