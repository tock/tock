// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::fmt::Write;
use core::panic::PanicInfo;
use kernel::ErrorCode;

use kernel::debug;
use kernel::debug::IoWrite;
use kernel::hil::led;
use kernel::hil::uart::Transmit;
use kernel::hil::uart::{self};
use kernel::static_init;
use kernel::utilities::cells::MapCell;
use kernel::utilities::cells::TakeCell;
use kernel::utilities::cells::VolatileCell;
use nrf52840::gpio::Pin;

const BUF_LEN: usize = 512;

struct DummyUsbClient {
    fired: VolatileCell<bool>,
}

impl DummyUsbClient {
    fn new() -> Self {
        Self {
            fired: VolatileCell::new(false),
        }
    }

    fn fired(&self) -> bool {
        self.fired.get()
    }

    fn clear(&self) {
        self.fired.set(false)
    }
}

impl uart::TransmitClient for DummyUsbClient {
    fn transmitted_buffer(&self, _: &'static mut [u8], _: usize, _: Result<(), ErrorCode>) {
        self.fired.set(true);
    }
}

struct Writer {
    initialized: bool,
    buffer: TakeCell<'static, [u8]>,
    client: &'static DummyUsbClient,
}

impl Writer {
    fn new(client: &'static DummyUsbClient, buffer: &'static mut [u8]) -> Self {
        Self {
            initialized: false,
            buffer: TakeCell::new(buffer),
            client,
        }
    }
}

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

impl IoWrite for Writer {
    fn write(&mut self, buf: &[u8]) -> usize {
        if !self.initialized {
            self.initialized = true;
        }
        // Here we mimic a synchronous UART output by calling transmit_buffer
        // on the CDC stack and then spinning on USB interrupts until the transaction
        // is complete. If the USB or CDC stack panicked, this may fail. It will also
        // fail if the panic occurred prior to the USB connection being initialized.
        // In the latter case, the LEDs should still blink in the panic pattern.

        // spin so that if any USB DMA is ongoing it will finish
        // we should only need this on the first call to write()
        let mut i = 0;
        loop {
            i += 1;
            cortexm4::support::nop();
            if i > 10000 {
                break;
            }
        }

        // copy_from_slice() requires equal length slices
        // This will truncate any writes longer than BUF_LEN, but simplifies the
        // code. In practice, BUF_LEN=512 always seems sufficient for the size of
        // individual calls to write made by the panic handler.
        let mut max = BUF_LEN;
        if buf.len() < BUF_LEN {
            max = buf.len();
        }

        // If CDC_REF_FOR_PANIC is not yet set we panicked very early,
        // and not much we can do. Don't want to double fault,
        // so just return.
        super::CDC_REF_FOR_PANIC
            .get()
            .and_then(MapCell::get)
            .map(|cdc| {
                let usb = &mut cdc.controller();
                self.buffer.take().map(|buffer| {
                    buffer[..max].copy_from_slice(&buf[..max]);
                    cdc.set_transmit_client(self.client);
                    let _ = cdc.transmit_buffer(buffer, max);
                });

                loop {
                    if let Some(interrupt) = unsafe { cortexm4::nvic::next_pending() } {
                        if interrupt == 39 {
                            usb.handle_interrupt();
                        }
                        let n = unsafe { cortexm4::nvic::Nvic::new(interrupt) };
                        n.clear_pending();
                        n.enable();
                    }
                    if self.client.fired() {
                        // buffer finished transmitting, return so we can output additional
                        // messages when requested by the panic handler.
                        break;
                    }
                }
                self.client.clear();
            });

        buf.len()
    }
}

/// Default panic handler for the Adafruit CLUE nRF52480 Express Board.
///
/// We just use the standard default provided by the debug module in the kernel.
#[cfg(not(test))]
#[panic_handler]
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    let led_kernel_pin = &nrf52840::gpio::GPIOPin::new(Pin::P1_01);
    let led = &mut led::LedHigh::new(led_kernel_pin);
    let static_buf = static_init!([u8; BUF_LEN], [0; BUF_LEN]);
    let dummy_usb_client = static_init!(DummyUsbClient, DummyUsbClient::new());
    let mut writer = Writer::new(dummy_usb_client, static_buf);
    debug::panic_new(
        &mut [led],
        &mut writer,
        pi,
        &cortexm4::support::nop,
        crate::PANIC_RESOURCES.get(),
    )
}
