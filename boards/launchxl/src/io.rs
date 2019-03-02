use core::fmt::Write;
use core::panic::PanicInfo;
use cortexm4;
use kernel::debug;
use kernel::hil::led;
use kernel::hil::uart;

use crate::PROCESSES;

struct Writer {
    initialized: bool,
}

static mut WRITER: Writer = Writer { initialized: false };

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        let uart = unsafe {
            cc26x2::uart::UART::unsafe_new(cc26x2::uart::PeripheralNum::_0)
        };
        if !self.initialized {
            self.initialized = true;
            uart::Configure::configure(
                &uart,
                uart::Parameters {
                    baud_rate: 115200,
                    stop_bits: uart::StopBits::One,
                    parity: uart::Parity::None,
                    hw_flow_control: false,
                    width: uart::Width::Eight,
                },
            );
        }
        for c in s.bytes() {
            uart.write(c as u32);
            while !uart.tx_fifo_not_full() {}
        }
        Ok(())
    }
}

#[cfg(not(test))]
#[panic_handler]
#[no_mangle]
pub unsafe extern "C" fn panic_fmt(pi: &PanicInfo) -> ! {
    // 6 = Red led, 7 = Green led
    const LED_PIN: usize = 6;

    let led = &mut led::LedLow::new(&mut cc26x2::gpio::PORT[LED_PIN]);
    let writer = &mut WRITER;
    debug::panic(&mut [led], writer, pi, &cortexm4::support::nop, &PROCESSES)
}
