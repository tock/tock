// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::fmt::Write;
use core::panic::PanicInfo;
use core::str;
use kernel::debug;
use kernel::debug::IoWrite;

struct Writer {}

static mut WRITER: Writer = Writer {};

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

impl IoWrite for Writer {
    fn write(&mut self, buf: &[u8]) -> usize {
        let uart = esp32::uart::Uart::new(esp32::uart::UART0_BASE);
        uart.disable_tx_interrupt();
        uart.disable_rx_interrupt();
        uart.transmit_sync(buf);
        buf.len()
    }
}

/// Panic handler.
#[cfg(not(test))]
#[panic_handler]
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    use core::ptr::addr_of_mut;

    let writer = &mut *addr_of_mut!(WRITER);

    debug::panic_print(
        writer,
        pi,
        &rv32i::support::nop,
        crate::PANIC_RESOURCES.get(),
    );

    loop {
        rv32i::support::nop();
    }
}

#[cfg(test)]
#[panic_handler]
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    use core::ptr::addr_of_mut;

    let writer = &mut *addr_of_mut!(WRITER);

    debug::panic_print(
        writer,
        pi,
        &rv32i::support::nop,
        crate::PANIC_RESOURCES.get(),
    );

    let _ = writeln!(writer, "{}", pi);
    loop {}
}
