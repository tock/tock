// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use core::fmt::Write;
use kernel::utilities::io_write::IoWrite;
use nxp_s32g3::linflexd::transmit_lf0_sync;

/// Writer is used by kernel::debug to panic message to the serial port.
pub struct Writer {}

/// Global static for debug writer
pub static mut WRITER: Writer = Writer {};

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        transmit_lf0_sync(s.as_bytes());
        Ok(())
    }
}

impl IoWrite for Writer {
    fn write(&mut self, buf: &[u8]) -> usize {
        transmit_lf0_sync(buf)
    }
}
#[cfg(not(test))]
#[panic_handler]
unsafe fn panic_handler(panic_info: &core::panic::PanicInfo) -> ! {
    use core::ptr::addr_of_mut;
    use kernel::debug;
    let writer = &mut *addr_of_mut!(WRITER);
    debug::panic_print_old(
        writer,
        panic_info,
        &cortexm7::support::nop,
        crate::PANIC_RESOURCES.get(),
    );
    loop {}
}
