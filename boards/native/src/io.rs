use core::fmt::*;
use core::panic::PanicInfo;
use core::str;
use kernel::debug;

extern crate tock_native_arch;

pub struct Writer {
    initialized: bool,
}

pub static mut WRITER: Writer = Writer { initialized: false };

impl Write for Writer {
    fn write_str(&mut self, _s: &str) -> ::core::fmt::Result {
        unimplemented!("write_str");
    }
}

/// Panic handler.
#[cfg(not(test))]
#[no_mangle]
#[panic_implementation]
pub unsafe extern "C" fn panic_fmt(panic_info: &PanicInfo) -> ! {
    let writer = &mut WRITER;

    //debug::panic(led, writer, pi, &tock_native_arch::nop)

    debug::panic_begin(&tock_native_arch::nop);
    debug::panic_banner(writer, panic_info);
    // Flush debug buffer if needed
    debug::flush(writer);
    debug::panic_process_info(writer);
    loop {}
}

