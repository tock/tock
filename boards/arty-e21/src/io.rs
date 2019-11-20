use arty_e21;
use core::fmt::Write;
use core::panic::PanicInfo;
use core::str;
use kernel::debug;
use kernel::hil::gpio;
use kernel::hil::led;
use rv32i;

use crate::PROCESSES;

struct Writer;

static mut WRITER: Writer = Writer {};

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        debug!("{}", s);
        Ok(())
    }
}

/// Panic handler.
#[cfg(not(test))]
#[no_mangle]
#[panic_handler]
pub unsafe extern "C" fn panic_fmt(pi: &PanicInfo) -> ! {
    // turn off the non panic leds, just in case
    let led_green = &arty_e21::gpio::PORT[19];
    gpio::Pin::make_output(led_green);
    gpio::Pin::set(led_green);

    let led_blue = &arty_e21::gpio::PORT[21];
    gpio::Pin::make_output(led_blue);
    gpio::Pin::set(led_blue);

    let led_red = &mut led::LedLow::new(&mut arty_e21::gpio::PORT[22]);
    let writer = &mut WRITER;
    debug::panic(&mut [led_red], writer, pi, &rv32i::support::nop, &PROCESSES)
}
