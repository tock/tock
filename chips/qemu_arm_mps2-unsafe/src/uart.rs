// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Panic writer over the CMSDK APB UART.

use kernel::hil;
use kernel::utilities::StaticRef;
use kernel::utilities::io_write::IoWrite;
use qemu_arm_mps2::uart::{Uart, UartRegisters};

/// A synchronous, polling writer for panic messages.
///
/// This bypasses all of [`Uart`]'s interrupt-driven state and is only ever
/// used from the panic handler.
pub struct UartPanicWriter<'a> {
    inner: Uart<'a>,
}

impl IoWrite for UartPanicWriter<'_> {
    fn write(&mut self, buf: &[u8]) -> usize {
        self.inner.transmit_sync(buf);
        buf.len()
    }
}

impl core::fmt::Write for UartPanicWriter<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

pub struct UartPanicWriterConfig {
    pub base: StaticRef<UartRegisters>,
    pub params: hil::uart::Parameters,
}

impl kernel::platform::chip::PanicWriter for UartPanicWriter<'_> {
    type Config = UartPanicWriterConfig;

    unsafe fn create_panic_writer(config: Self::Config) -> impl IoWrite + core::fmt::Write {
        use hil::uart::Configure as _;

        let inner = Uart::new(config.base);
        inner.reset();
        // Nothing to report a failure to: this runs from the panic handler,
        // and the parameters come from the board's own configuration.
        let _ = inner.configure(config.params);
        UartPanicWriter { inner }
    }
}
