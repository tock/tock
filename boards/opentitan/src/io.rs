use core::fmt::Write;
use core::panic::PanicInfo;
use core::str;
use kernel::debug;
use kernel::debug::IoWrite;
use kernel::hil::gpio;
use kernel::hil::led;

use crate::CHIP;
use crate::PROCESSES;

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
            earlgrey::uart::UART0.transmit_sync(buf);
        }
    }
}

/// Panic handler.
#[cfg(not(test))]
#[no_mangle]
#[panic_handler]
pub unsafe extern "C" fn panic_fmt(pi: &PanicInfo) -> ! {
    // turn off the non panic leds, just in case
    let first_led = &mut led::LedLow::new(&mut earlgrey::gpio::PORT[7]);
    gpio::Pin::make_output(&earlgrey::gpio::PORT[7]);

    let writer = &mut WRITER;

    debug::panic(
        &mut [first_led],
        writer,
        pi,
        &riscv::support::nop,
        &PROCESSES,
        &CHIP,
    )
}
