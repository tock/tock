// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright OxidOS Automotive 2026.

use core::fmt::Write;
use core::panic::PanicInfo;
use core::ptr::addr_of_mut;

use kernel::debug;
use kernel::debug::IoWrite;

/// Writer is used by kernel::debug to print messages to the serial port.
pub struct Writer {}

/// Global static for debug writer
#[no_mangle]
pub static mut WRITER: Writer = Writer {};

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

impl IoWrite for Writer {
    fn write(&mut self, buf: &[u8]) -> usize {
        let uart = stm32u545::usart::Usart::new(stm32u545::usart::USART1_BASE);

        for &c in buf {
            uart.transmit_byte(c);
        }
        buf.len()
    }
}

/// Panic handler.
#[panic_handler]
pub unsafe fn panic_fmt(info: &PanicInfo) -> ! {
    let writer = &mut *addr_of_mut!(WRITER);

    debug::panic_print(
        writer,
        info,
        &cortexm33::support::nop,
        crate::PANIC_RESOURCES.get(),
    );

    loop {}
}
