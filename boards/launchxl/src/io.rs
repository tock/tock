use cc26xx;
use core::fmt::{Arguments, Write};
use kernel::debug;
use kernel::hil::led;

struct Writer {}

static mut WRITER: Writer = Writer {};

impl Write for Writer {
    fn write_str(&mut self, _s: &str) -> ::core::fmt::Result {
        Ok(())
    }
}

#[cfg(not(test))]
#[lang = "panic_fmt"]
#[no_mangle]
pub unsafe extern "C" fn rust_begin_unwind(args: Arguments, file: &'static str, line: u32) -> ! {
    // 6 = Red led, 7 = Green led
    const LED_PIN: usize = 6;

    let led = &mut led::LedLow::new(&mut cc26xx::gpio::PORT[LED_PIN]);
    let writer = &mut WRITER;
    debug::panic(led, writer, args, file, line)
}
