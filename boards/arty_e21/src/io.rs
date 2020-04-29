use arty_e21_chip;
use core::fmt::Write;
use core::panic::PanicInfo;
use core::str;
use kernel::debug;
use kernel::debug::IoWrite;
use kernel::hil::gpio;
use kernel::hil::led;
use rv32i;

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
            arty_e21_chip::uart::UART0.transmit_sync(buf);
        }
    }
}

/// Panic handler.
#[cfg(not(test))]
#[no_mangle]
#[panic_handler]
pub unsafe extern "C" fn panic_fmt(pi: &PanicInfo) -> ! {
    // turn off the non panic leds, just in case
    let led_green = &arty_e21_chip::gpio::PORT[1];
    gpio::Pin::make_output(led_green);
    gpio::Pin::clear(led_green);

    let led_blue = &arty_e21_chip::gpio::PORT[0];
    gpio::Pin::make_output(led_blue);
    gpio::Pin::clear(led_blue);

    let led_red = &mut led::LedHigh::new(&mut arty_e21_chip::gpio::PORT[2]);
    let writer = &mut WRITER;
    debug::panic(
        &mut [led_red],
        writer,
        pi,
        &rv32i::support::nop,
        &PROCESSES,
        &CHIP,
    )
}
