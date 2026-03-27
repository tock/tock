// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

use core::fmt::Write;
use kernel::debug::IoWrite;

struct Writer {}

static mut WRITER: Writer = Writer {};

impl Write for Writer {
    fn write_str(&mut self, _s: &str) -> core::fmt::Result {
        Ok(())
    }
}

impl IoWrite for Writer {
    fn write(&mut self, buf: &[u8]) -> usize {
        buf.len()
    }
}

#[cfg(not(test))]
#[panic_handler]
pub unsafe fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
