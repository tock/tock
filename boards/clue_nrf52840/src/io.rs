// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::fmt::Write;
use core::panic::PanicInfo;
use core::ptr::addr_of;
use core::ptr::addr_of_mut;
use kernel::ErrorCode;

use kernel::debug;
use kernel::debug::IoWrite;
use kernel::hil::led;
use kernel::hil::uart::Transmit;
use kernel::hil::uart::{self};
use kernel::utilities::cells::VolatileCell;
use nrf52840::gpio::Pin;

use crate::CHIP;
use crate::PROCESSES;
use crate::PROCESS_PRINTER;

struct Writer {
    initialized: bool,
}

static mut WRITER: Writer = Writer { initialized: false };

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

const BUF_LEN: usize = 512;
static mut STATIC_PANIC_BUF: [u8; BUF_LEN] = [0; BUF_LEN];

static mut DUMMY: DummyUsbClient = DummyUsbClient {
    fired: VolatileCell::new(false),
};

struct DummyUsbClient {
    fired: VolatileCell<bool>,
}

impl uart::TransmitClient for DummyUsbClient {
    fn transmitted_buffer(&self, _: &'static mut [u8], _: usize, _: Result<(), ErrorCode>) {
        self.fired.set(true);
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

        unsafe {
            // If CDC_REF_FOR_PANIC is not yet set we panicked very early,
            // and not much we can do. Don't want to double fault,
            // so just return.
            super::CDC_REF_FOR_PANIC.map(|cdc| {
                // Lots of unsafe dereferencing of global static mut objects here.
                // However, this should be okay, because it all happens within
                // a single thread, and:
                // - This is the only place the global CDC_REF_FOR_PANIC is used, the logic is the same
                //   as applies for the global CHIP variable used in the panic handler.
                // - We do create multiple mutable references to the STATIC_PANIC_BUF, but we never
                //   access the STATIC_PANIC_BUF after a slice of it is passed to transmit_buffer
                //   until the slice has been returned in the uart callback.
                // - Similarly, only this function uses the global DUMMY variable, and we do not
                //   mutate it.
                let usb = &mut cdc.controller();
                STATIC_PANIC_BUF[..max].copy_from_slice(&buf[..max]);
                let static_buf = &mut *addr_of_mut!(STATIC_PANIC_BUF);
                cdc.set_transmit_client(&*addr_of!(DUMMY));
                let _ = cdc.transmit_buffer(static_buf, max);
                loop {
                    if let Some(interrupt) = cortexm4::nvic::next_pending() {
                        if interrupt == 39 {
                            usb.handle_interrupt();
                        }
                        let n = cortexm4::nvic::Nvic::new(interrupt);
                        n.clear_pending();
                        n.enable();
                    }
                    if (*addr_of!(DUMMY)).fired.get() {
                        // buffer finished transmitting, return so we can output additional
                        // messages when requested by the panic handler.
                        break;
                    }
                }
                (*addr_of!(DUMMY)).fired.set(false);
            });
        }
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
    let writer = &mut *addr_of_mut!(WRITER);
    debug::panic(
        &mut [led],
        writer,
        pi,
        &cortexm4::support::nop,
        &*addr_of!(PROCESSES),
        &*addr_of!(CHIP),
        &*addr_of!(PROCESS_PRINTER),
    )
}
