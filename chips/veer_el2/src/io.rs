// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2024 Antmicro <www.antmicro.com>

use core::fmt::Write;
use core::ptr::write_volatile;
use kernel::debug::IoWrite;
use kernel::hil;
use kernel::ErrorCode;

pub struct Writer {}
static mut WRITER: Writer = Writer {};

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

impl IoWrite for Writer {
    fn write(&mut self, buf: &[u8]) -> usize {
        for b in buf {
            // Print to this address for simulation output
            unsafe {
                write_volatile(0xd0580000 as *mut u32, (*b) as u32);
            }
        }
        buf.len()
    }
}

pub struct SemihostUart {}

impl SemihostUart {
    pub fn new() -> SemihostUart {
        SemihostUart {}
    }
}

impl Default for SemihostUart {
    fn default() -> Self {
        Self::new()
    }
}

impl hil::uart::Configure for SemihostUart {
    fn configure(&self, _params: hil::uart::Parameters) -> Result<(), ErrorCode> {
        Ok(())
    }
}

impl<'a> hil::uart::Transmit<'a> for SemihostUart {
    fn set_transmit_client(&self, _client: &'a dyn hil::uart::TransmitClient) {}

    fn transmit_buffer(
        &self,
        tx_buffer: &'static mut [u8],
        tx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        unsafe {
            WRITER.write(&tx_buffer[..tx_len]);
        }
        // Returning Ok(()) requires an async confirmation of the transfer which is supposed to happen later on.
        // We have no interrupts here and nothing happens asynchronously so just write all the bytes immediately
        // and pretend it failed.
        Err((ErrorCode::FAIL, tx_buffer))
    }
    fn transmit_word(&self, _word: u32) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn transmit_abort(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }
}

impl<'a> hil::uart::Receive<'a> for SemihostUart {
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
