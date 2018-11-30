use core::fmt::Write;
use core::panic::PanicInfo;
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
        let uart = unsafe { &mut nrf52::uart::UARTE0 };
        if !self.initialized {
            self.initialized = true;
            uart.configure(uart::UARTParameters {
                baud_rate: 115200,
                stop_bits: uart::StopBits::One,
                parity: uart::Parity::None,
                hw_flow_control: false,
            });
        }
        for c in s.bytes() {
            unsafe {
                uart.send_byte(c);
            }
            while !uart.tx_ready() {}
        }
        Ok(())
    }
}

#[cfg(not(test))]
#[no_mangle]
#[panic_handler]
/// Panic handler
pub unsafe extern "C" fn panic_fmt(pi: &PanicInfo) -> ! {
    // The nRF52 DK LEDs (see back of board)
    const LED1_PIN: usize = 17;
    let led = &mut led::LedLow::new(&mut nrf5x::gpio::PORT[LED1_PIN]);
    let writer = &mut WRITER;
    debug::panic(&mut [led], writer, pi, &cortexm4::support::nop, &PROCESSES)
}
