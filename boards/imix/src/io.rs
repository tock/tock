use core::fmt::Write;
use core::panic::PanicInfo;
use cortexm4;
use kernel::debug;
use kernel::hil::led;
use kernel::hil::uart::{self, Configure};
use sam4l;

use PROCESSES;

struct Writer {
    initialized: bool,
}

static mut WRITER: Writer = Writer { initialized: false };

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        let uart = unsafe { &mut sam4l::usart::USART3 };
        let regs_manager = &sam4l::usart::USARTRegManager::panic_new(&uart);
        if !self.initialized {
            self.initialized = true;
            uart.configure(uart::Parameters {
                baud_rate: 115200,
                width: uart::Width::Eight,
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
#[panic_implementation]
pub unsafe extern "C" fn panic_fmt(pi: &PanicInfo) -> ! {
    let led = &mut led::LedLow::new(&mut sam4l::gpio::PC[10]);
    let writer = &mut WRITER;
    debug::panic(&mut [led], writer, pi, &cortexm4::support::nop, &PROCESSES)
}
