// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2024 Antmicro <www.antmicro.com>

use core::cell::Cell;
use core::ptr::write_volatile;
use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil;
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::ErrorCode;

pub struct SemihostUart<'a> {
    deferred_call: DeferredCall,
    tx_client: OptionalCell<&'a dyn hil::uart::TransmitClient>,
    tx_buffer: TakeCell<'static, [u8]>,
    tx_len: Cell<usize>,
}

impl<'a> SemihostUart<'a> {
    pub fn new() -> SemihostUart<'a> {
        SemihostUart {
            deferred_call: DeferredCall::new(),
            tx_client: OptionalCell::empty(),
            tx_buffer: TakeCell::empty(),
            tx_len: Cell::new(0),
        }
    }
}

impl Default for SemihostUart<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl hil::uart::Configure for SemihostUart<'_> {
    fn configure(&self, _params: hil::uart::Parameters) -> Result<(), ErrorCode> {
        Ok(())
    }
}

impl<'a> hil::uart::Transmit<'a> for SemihostUart<'a> {
    fn set_transmit_client(&self, client: &'a dyn hil::uart::TransmitClient) {
        self.tx_client.set(client);
    }

    fn transmit_buffer(
        &self,
        tx_buffer: &'static mut [u8],
        tx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if tx_len == 0 || tx_len > tx_buffer.len() {
            Err((ErrorCode::SIZE, tx_buffer))
        } else if self.tx_buffer.is_some() {
            Err((ErrorCode::BUSY, tx_buffer))
        } else {
            for b in &tx_buffer[..tx_len] {
                unsafe {
                    // Print to this address for simulation output
                    write_volatile(0xd0580000 as *mut u32, (*b) as u32);
                }
            }
            self.tx_len.set(tx_len);
            self.tx_buffer.replace(tx_buffer);
            // The whole buffer was transmited immediately
            self.deferred_call.set();
            Ok(())
        }
    }

    fn transmit_word(&self, _word: u32) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn transmit_abort(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }
}

impl<'a> hil::uart::Receive<'a> for SemihostUart<'a> {
    fn set_receive_client(&self, _client: &'a dyn hil::uart::ReceiveClient) {}
    fn receive_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        _rx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        Err((ErrorCode::FAIL, rx_buffer))
    }
    fn receive_word(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }
    fn receive_abort(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }
}

impl DeferredCallClient for SemihostUart<'_> {
    fn register(&'static self) {
        self.deferred_call.register(self);
    }

    fn handle_deferred_call(&self) {
        self.tx_client.map(|client| {
            self.tx_buffer.take().map(|tx_buf| {
                client.transmitted_buffer(tx_buf, self.tx_len.get(), Ok(()));
            });
        });
    }
}
