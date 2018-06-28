use core::fmt::Write;
use std::panic;

use kernel::debug;

extern crate tock_native_arch;

pub struct Writer {
    initialized: bool,
}

pub static mut WRITER: Writer = Writer { initialized: false };

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        eprint!("{}", s);
        Ok(())
    }
}

/// Panic handler.
pub fn panic_hook(panic_info: &panic::PanicInfo) {

    //debug::panic(led, writer, pi, &tock_native_arch::nop)

    unsafe {
        let writer = &mut WRITER;

        debug::panic_begin(&tock_native_arch::nop);
        debug::panic_banner(writer, panic_info);
        // Flush debug buffer if needed
        debug::flush(writer);
        debug::panic_process_info(writer);
    }
}

