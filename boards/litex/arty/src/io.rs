use core::fmt::Write;
use core::panic::PanicInfo;
use core::str;
use kernel::debug;
use kernel::debug::IoWrite;
use rv32i;

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
    fn write(&mut self, buf: &[u8]) {
        unsafe {
            PANIC_REFERENCES.uart.unwrap().transmit_sync(buf);
        }
    }
}

/// Panic handler.
#[cfg(not(test))]
#[no_mangle]
#[panic_handler]
pub unsafe extern "C" fn panic_fmt(pi: &PanicInfo) -> ! {
    let panic_led = PANIC_REFERENCES
        .led_controller
        .and_then(|ctrl| ctrl.panic_led(0));

    let writer = &mut WRITER;

    debug::panic(
        &mut [&mut panic_led.unwrap()],
        writer,
        pi,
        &rv32i::support::nop,
        &PROCESSES,
        &PANIC_REFERENCES.chip,
    )
}
