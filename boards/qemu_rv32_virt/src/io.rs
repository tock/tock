// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::fmt::Write;
use core::panic::PanicInfo;
use core::str;

use kernel::debug;
use kernel::debug::IoWrite;

use crate::CHIP;
use crate::PROCESSES;
use crate::PROCESS_PRINTER;

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
        let uart = qemu_rv32_virt_chip::uart::Uart16550::new(qemu_rv32_virt_chip::uart::UART0_BASE);
        uart.transmit_sync(buf);
        buf.len()
    }
}

/// Panic handler.
#[cfg(not(test))]
#[panic_handler]
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    use core::ptr::{addr_of, addr_of_mut};

    let writer = &mut *addr_of_mut!(WRITER);

    debug::panic_print::<_, _, _>(
        writer,
        pi,
        &rv32i::support::nop,
        &*addr_of!(PROCESSES),
        &*addr_of!(CHIP),
        &*addr_of!(PROCESS_PRINTER),
    );

    // The system is no longer in a well-defined state. Use
    // semihosting commands to exit QEMU with a return code of 1.
    rv32i::semihost_command(0x18, 1, 0);

    // To satisfy the ! return type constraints.
    loop {}
}
