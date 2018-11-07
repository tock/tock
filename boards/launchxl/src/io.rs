use cc26x2;
use core::fmt::Write;
use core::panic::PanicInfo;
use cortexm4;
use kernel::debug;
use kernel::hil::led;
use kernel::hil::uart;

use PROCESSES;

struct Writer {
    initialized: bool,
}

static mut WRITER: Writer = Writer { initialized: false };

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        let uart = unsafe { &mut cc26x2::uart::UART0 };
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
            uart.send_byte(c);
            while !uart.tx_fifo_not_full() {}
        }
        Ok(())
    }
}

#[cfg(not(test))]
#[panic_implementation]
#[no_mangle]
pub unsafe extern "C" fn panic_fmt(pi: &PanicInfo) -> ! {
    // 6 = Red led, 7 = Green led
    const LED_PIN: usize = 6;

    let led = &mut led::LedLow::new(&mut cc26x2::gpio::PORT[LED_PIN]);
    let writer = &mut WRITER;
    debug::panic(&mut [led], writer, pi, &cortexm4::support::nop, &PROCESSES)
}
