// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! QEMU's memory mapped 16550 UART

use kernel::hil;
use kernel::utilities::io_write::IoWrite;
use kernel::utilities::StaticRef;
use qemu_virt_chip::uart::Uart16550;
use qemu_virt_chip::uart::Uart16550Registers;

pub const UART0_BASE: StaticRef<Uart16550Registers> =
    unsafe { StaticRef::new(0x1000_0000 as *const Uart16550Registers) };

/// A synchronous writer for the QEMU RV32 useful for panics.
///
/// For boards that want to use the UART to display panic messages, this
/// provides an implementation of
/// [`PanicWriter`](kernel::platform::chip::PanicWriter) with synchronous
/// output.
///
/// This is only to be used by panic messages and is not used within the normal
/// operation of the Tock kernel.
///
/// TODO: Validate this [`UartPanicWriter`] is always sound to create.
pub struct UartPanicWriter<'a> {
    inner: Uart16550<'a>,
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

/// Configuration for the synchronous UART panic writer.
///
/// This captures everything needed to setup the UART for panic display, even
/// if the normal kernel had initialized it differently.
pub struct UartPanicWriterConfig {
    pub params: hil::uart::Parameters,
}

impl kernel::platform::chip::PanicWriter for UartPanicWriter<'_> {
    type Config = UartPanicWriterConfig;

    unsafe fn create_panic_writer(config: Self::Config) -> impl IoWrite + core::fmt::Write {
        use hil::uart::Configure as _;

        let inner = Uart16550::new(UART0_BASE);
        let _ = inner.configure(config.params);
        UartPanicWriter { inner }
    }
}
