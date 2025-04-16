// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::fmt::Write;
use core::panic::PanicInfo;
use core::ptr::write_volatile;
use core::ptr::{addr_of, addr_of_mut};
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
                write_volatile(0xd0580000 as *mut u32, (*b) as u32);
            }
        }
        buf.len()
    }
}

/// Panic handler.
///
/// # Safety
/// Accesses memory-mapped registers.
#[cfg(not(test))]
#[panic_handler]
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    let writer = &mut *addr_of_mut!(WRITER);

    debug::panic_print(
        writer,
        pi,
        &rv32i::support::nop,
        &*addr_of!(PROCESSES),
        &*addr_of!(CHIP),
        &*addr_of!(PROCESS_PRINTER),
    );

    // By writing 0xff to this address we can exit the simulation.
    // So instead of blinking in a loop let's exit the simulation.
    write_volatile(0xd0580000 as *mut u8, 0xff);

    unreachable!()
}
