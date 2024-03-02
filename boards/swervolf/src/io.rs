// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::fmt::Write;
use core::panic::PanicInfo;
use core::ptr::write_volatile;
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
        for b in buf {
            // Print to a special address for simulation output
            unsafe {
                write_volatile(0x8000_1008 as *mut u8, *b);
            }
        }
        buf.len()
    }
}

/// Panic handler.
#[cfg(not(test))]
#[no_mangle]
#[panic_handler]
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    let writer = &mut WRITER;

    debug::panic_print(
        writer,
        pi,
        &rv32i::support::nop,
        &PROCESSES,
        &CHIP,
        &PROCESS_PRINTER,
    );

    // By writing to address 0x80001009 we can exit the simulation.
    // So instead of blinking in a loop let's exit the simulation.
    write_volatile(0x8000_1009 as *mut u8, 20);

    unreachable!()
}
