// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::fmt::Write;
use core::panic::PanicInfo;
use core::str;
use kernel::debug;
use kernel::debug::IoWrite;

use crate::{PANIC_REFERENCES, PROCESSES};

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
        unsafe {
            PANIC_REFERENCES.uart.unwrap().transmit_sync(buf);
        }
        buf.len()
    }
}

/// Panic handler.
#[cfg(not(test))]
#[panic_handler]
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    use core::ptr::{addr_of, addr_of_mut};

    let panic_led = PANIC_REFERENCES
        .led_controller
        .and_then(|ctrl| ctrl.panic_led(0));

    let writer = &mut *addr_of_mut!(WRITER);

    debug::panic(
        &mut [&mut panic_led.unwrap()],
        writer,
        pi,
        &rv32i::support::nop,
        &*addr_of!(PROCESSES),
        &*addr_of!(PANIC_REFERENCES.chip),
        &*addr_of!(PANIC_REFERENCES.process_printer),
    )
}
