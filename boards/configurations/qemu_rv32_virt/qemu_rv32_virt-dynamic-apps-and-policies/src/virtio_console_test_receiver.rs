// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Test [`hil::uart::ReceiveClient`] for the board's virtio console: prints
//! each byte it receives via `kernel::debug!()`, then immediately starts
//! listening for the next one.
//!
//! This exists purely to demonstrate/exercise the virtio console UART
//! implementation from the kernel; it is not wired to any userspace
//! interface.

use kernel::ErrorCode;
use kernel::hil;
use kernel::utilities::cells::OptionalCell;

pub struct VirtioConsoleTestReceiver {
    uart: OptionalCell<&'static dyn hil::uart::Receive<'static>>,
}

impl VirtioConsoleTestReceiver {
    pub const fn new() -> Self {
        Self {
            uart: OptionalCell::empty(),
        }
    }

    /// Start listening: register as `uart`'s receive client and issue the
    /// first one-byte read.
    pub fn start(&self, uart: &'static dyn hil::uart::Receive<'static>, buffer: &'static mut [u8; 1]) {
        self.uart.set(uart);
        if let Err((err, _buffer)) = uart.receive_buffer(buffer, 1) {
            kernel::debug!("virtio console: failed to start receiving: {:?}", err);
        }
    }
}

impl hil::uart::ReceiveClient for VirtioConsoleTestReceiver {
    fn received_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
        rval: Result<(), ErrorCode>,
        error: hil::uart::Error,
    ) {
        if rval.is_ok() && rx_len >= 1 {
            let byte = rx_buffer[0];
            kernel::debug!(
                "virtio console rx: {:#04x} ({:?})",
                byte,
                byte as char
            );
        } else {
            kernel::debug!("virtio console: receive error: {:?} {:?}", rval, error);
        }

        // Keep listening.
        self.uart.map(|uart| {
            if let Err((err, _buffer)) = uart.receive_buffer(rx_buffer, 1) {
                kernel::debug!("virtio console: failed to re-arm receive: {:?}", err);
            }
        });
    }
}
