use core::fmt::Write;
use core::panic::PanicInfo;
use core::str;
use cortexm4;
use kernel::debug;
use kernel::hil::led;
use kernel::hil::uart::{self, UART};

use PROCESSES;

struct Writer {
    initialized: bool,
}

static mut WRITER: Writer = Writer { initialized: false };

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        let uart = unsafe { &mut sam4l::usart::USART0 };
        let regs_manager = &sam4l::usart::USARTRegManager::panic_new(&uart);
        if !self.initialized {
            self.initialized = true;
            uart.configure(uart::UARTParameters {
                baud_rate: 115200,
                stop_bits: uart::StopBits::One,
                parity: uart::Parity::None,
                hw_flow_control: false,
            });
            uart.enable_tx(regs_manager);
        }
        // XXX: I'd like to get this working the "right" way, but I'm not sure how
        for c in s.bytes() {
            uart.send_byte(regs_manager, c);
            while !uart.tx_ready(regs_manager) {}
        }
        Ok(())
    }
}

/// Panic handler.
#[cfg(not(test))]
#[no_mangle]
#[panic_handler]
pub unsafe extern "C" fn panic_fmt(pi: &PanicInfo) -> ! {
    // turn off the non panic leds, just in case
    let led_green = &sam4l::gpio::PA[14];
    led_green.enable_output();
    led_green.set();
    let led_blue = &sam4l::gpio::PA[15];
    led_blue.enable_output();
    led_blue.set();

    let led_red = &mut led::LedLow::new(&mut sam4l::gpio::PA[13]);
    let writer = &mut WRITER;
    debug::panic(
        &mut [led_red],
        writer,
        pi,
        &cortexm4::support::nop,
        &PROCESSES,
    )
}
