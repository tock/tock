// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Support for printing panic messages over CDC-ACM-over-USB.
//!
//! This synchronously uses the CDC stack to print panic data.

use core::cell::Cell;
use core::fmt::Write;
use core::panic::PanicInfo;

use capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm;
use capsules_extra::usb::cdc::CdcAcm;
use kernel::ErrorCode;
use kernel::hil::uart::{self, Transmit};
use kernel::platform::chip::PanicWriter;
use kernel::utilities::cells::TakeCell;
use kernel::utilities::io_write::IoWrite;

/// Size of the buffer used to hold panic output before handing it to the USB
/// stack.
pub const BUF_LEN: usize = 512;

/// NVIC interrupt number for the nRF52840's USBD peripheral.
const USBD_INTERRUPT: u32 = 39;

type CdcAlarm = VirtualMuxAlarm<'static, nrf52::rtc::Rtc<'static>>;

/// Concrete CDC-ACM-over-USB driver type used by nRF52 boards for panic
/// output.
pub type CdcDriver = CdcAcm<'static, nrf52::usbd::Usbd<'static>, CdcAlarm>;

/// All of the state needed to print panic output over a [`CdcDriver`].
pub struct CdcPanicWriterData {
    cdc: &'static CdcDriver,
    buf: TakeCell<'static, [u8]>,
    fired: Cell<bool>,
}

impl CdcPanicWriterData {
    /// Create a new `CdcPanicWriterData` for `cdc`, buffering panic output in
    /// `buf` (which should be [`BUF_LEN`] bytes).
    pub fn new(cdc: &'static CdcDriver, buf: &'static mut [u8]) -> Self {
        Self {
            cdc,
            buf: TakeCell::new(buf),
            fired: Cell::new(false),
        }
    }
}

impl uart::TransmitClient for CdcPanicWriterData {
    fn transmitted_buffer(&self, buffer: &'static mut [u8], _: usize, _: Result<(), ErrorCode>) {
        // The transaction is done, so the buffer belongs to us again.
        self.buf.put(Some(buffer));
        self.fired.set(true);
    }
}

/// Synchronous [`IoWrite`]/[`Write`] implementation that mimics a blocking
/// UART by driving the CDC-ACM-over-USB stack by hand and polling for
/// interrupts, for use while panicking.
struct CdcPanicWriter {
    data: &'static CdcPanicWriterData,
}

impl Write for CdcPanicWriter {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

impl IoWrite for CdcPanicWriter {
    fn write(&mut self, buf: &[u8]) -> usize {
        let data = self.data;
        let cdc = data.cdc;

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

        let usb = &mut cdc.controller();
        // If the panic buffer is still out on loan (e.g. a reentrant panic)
        // there is not much we can do. Don't want to double fault, so just
        // return.
        if let Some(static_buf) = data.buf.take() {
            static_buf[..max].copy_from_slice(&buf[..max]);
            match cdc.transmit_buffer(static_buf, max) {
                Ok(()) => {
                    // Only look for (and service) the USBD interrupt here.
                    // `next_pending()` reports the lowest-numbered pending
                    // interrupt system-wide, and once we're panicking
                    // nothing else is running the normal kernel loop to
                    // service other peripherals (e.g. the RTC driving the
                    // console's alarm). If one of those is left with an
                    // unserviced, permanently-pending interrupt, it has a
                    // lower number than USBD and would starve us from ever
                    // seeing USBD's own pending bit again. Masking
                    // everything else out avoids that entirely.
                    let ignore_all_but_usbd = (!0u128, !(1u128 << USBD_INTERRUPT));
                    loop {
                        if cortexm4::nvic::next_pending_with_mask(ignore_all_but_usbd).is_some() {
                            usb.handle_interrupt();
                            let n = cortexm4::nvic::Nvic::new(USBD_INTERRUPT);
                            n.clear_pending();
                            n.enable();
                        }
                        if data.fired.get() {
                            // buffer finished transmitting, return so we can output
                            // additional messages when requested by the panic handler.
                            break;
                        }
                    }
                    data.fired.set(false);
                }
                Err((_err, unsent_buf)) => {
                    // Transmission did not start, so return the buffer so the next
                    // call to write() can use it.
                    data.buf.put(Some(unsent_buf));
                }
            }
        }
        buf.len()
    }
}

impl PanicWriter for CdcPanicWriterData {
    type Config = &'static CdcPanicWriterData;

    fn create_panic_writer(data: Self::Config, _panic: &PanicInfo) -> impl IoWrite + Write {
        // Set the client for the CDC stack to us now.
        data.cdc.set_transmit_client(data);
        CdcPanicWriter { data }
    }
}
