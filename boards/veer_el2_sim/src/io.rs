// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::fmt::Write;
use core::panic::PanicInfo;
use core::ptr::write_volatile;
use kernel::debug;
use kernel::utilities::io_write::IoWrite;

struct Writer {}

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

impl kernel::platform::chip::PanicWriter for Writer {
    type Config = ();
    unsafe fn create_panic_writer(_config: Self::Config) -> impl IoWrite + core::fmt::Write {
        Writer {}
    }
}

/// Panic handler.
///
/// # Safety
/// Accesses memory-mapped registers.
#[cfg(not(test))]
#[panic_handler]
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    debug::panic_print::<Writer, _, _>(
        (),
        pi,
        &rv32i::support::nop,
        crate::PANIC_RESOURCES.get(),
    );

    // By writing 0xff to this address we can exit the simulation.
    // So instead of blinking in a loop let's exit the simulation.
    write_volatile(0xd0580000 as *mut u8, 0xff);

    unreachable!()
}
