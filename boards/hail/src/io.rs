use core::fmt::*;
use core::str;
use kernel::debug;
use kernel::hil::led;
use kernel::hil::uart::{self, UART};
use sam4l;

struct Writer {
    initialized: bool,
}

static mut WRITER: Writer = Writer { initialized: false };

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        let uart = unsafe { &mut sam4l::usart::USART0 };
        if !self.initialized {
            self.initialized = true;
            uart.init(uart::UARTParams {
                baud_rate: 115200,
                stop_bits: uart::StopBits::One,
                parity: uart::Parity::None,
                hw_flow_control: false,
            });
            uart.enable_tx();
        }
        // XXX: I'd like to get this working the "right" way, but I'm not sure how
        for c in s.bytes() {
            uart.send_byte(c);
            while !uart.tx_ready() {}
        }
        Ok(())
    }
}

/// Panic handler.
#[cfg(not(test))]
#[no_mangle]
#[lang = "panic_fmt"]
pub unsafe extern "C" fn panic_fmt(args: Arguments, file: &'static str, line: u32) -> ! {
    // turn off the non panic leds, just in case
    let ledg = &sam4l::gpio::PA[14];
    ledg.enable_output();
    ledg.set();
    let ledb = &sam4l::gpio::PA[15];
    ledb.enable_output();
    ledb.set();

    let led = &mut led::LedLow::new(&mut sam4l::gpio::PA[13]);
    let writer = &mut WRITER;
    debug::panic(led, writer, args, file, line)
}
