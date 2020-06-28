use core::fmt::Write;
use core::panic::PanicInfo;

use cortexm4;
use kernel::debug;
use kernel::debug::IoWrite;
use kernel::hil::led;
use kernel::hil::uart::{self, Configure};
use nrf52840::gpio::Pin;

use crate::CHIP;
use crate::PROCESSES;

struct Writer {
    initialized: bool,
}

static mut WRITER: Writer = Writer { initialized: false };

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

impl IoWrite for Writer {
    fn write(&mut self, buf: &[u8]) {
        let uart = unsafe { &mut nrf52840::uart::UARTE0 };
        if !self.initialized {
            self.initialized = true;
            uart.configure(uart::Parameters {
                baud_rate: 115200,
                stop_bits: uart::StopBits::One,
                parity: uart::Parity::None,
                hw_flow_control: false,
                width: uart::Width::Eight,
            });
        }
        for &c in buf {
            unsafe {
                uart.send_byte(c);
            }
            while !uart.tx_ready() {}
        }
    }
}

/// Default panic handler for the Nano 33 Board.
///
/// We just use the standard default provided by the debug module in the kernel.
#[cfg(not(test))]
#[no_mangle]
#[panic_handler]
pub unsafe extern "C" fn panic_fmt(pi: &PanicInfo) -> ! {
    const LED_KERNEL_PIN: Pin = Pin::P0_13;
    let led = &mut led::LedLow::new(&mut nrf52840::gpio::PORT[LED_KERNEL_PIN]);
    let writer = &mut WRITER;
    debug::panic(
        &mut [led],
        writer,
        pi,
        &cortexm4::support::nop,
        &PROCESSES,
        &CHIP,
    )
}
