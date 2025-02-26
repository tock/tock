// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Test reception on the virtualized UART: best if multiple Tests are
//! instantiated and tested in parallel.
use crate::virtualizers::virtual_uart::UartDevice;

use kernel::debug;
use kernel::hil::uart;
use kernel::hil::uart::Receive;
use kernel::utilities::cells::TakeCell;
use kernel::ErrorCode;

pub struct TestVirtualUartReceive<U: uart::Uart<'static> + 'static> {
    device: &'static UartDevice<'static, U>,
    buffer: TakeCell<'static, [u8]>,
}

impl<U: uart::Uart<'static> + 'static> TestVirtualUartReceive<U> {
    pub fn new(device: &'static UartDevice<'static, U>, buffer: &'static mut [u8]) -> Self {
        Self {
            device,
            buffer: TakeCell::new(buffer),
        }
    }

    pub fn run(&self) {
        let buf = self.buffer.take().unwrap();
        let len = buf.len();
        debug!("Starting receive of length {}", len);
        self.device
            .receive_buffer(buf, len)
            .expect("Calling receive_buffer() in virtual_uart test failed");
    }
}

impl<U: uart::Uart<'static> + 'static> uart::ReceiveClient for TestVirtualUartReceive<U> {
    fn received_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
        rcode: Result<(), ErrorCode>,
        _error: uart::Error,
    ) {
        debug!("Virtual uart read complete: {:?}: ", rcode);
        for i in 0..rx_len {
            debug!("{:02x} ", rx_buffer[i]);
        }
        debug!("Starting receive of length {}", rx_len);
        self.device
            .receive_buffer(rx_buffer, rx_len)
            .expect("Calling receive_buffer() in virtual_uart test failed");
    }
}
