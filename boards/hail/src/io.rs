use core::fmt::Write;
use core::panic::PanicInfo;
use core::str;
use cortexm4;
use kernel::debug;
use kernel::debug::IoWrite;
use kernel::hil::led;
use kernel::hil::uart::{self, Configure};

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
        // Here, we create a second instance of the USART0 struct.
        // This is okay because we only call this during a panic, and
        // we will never actually process the interrupts
        let uart = unsafe { sam4l::usart::USART::new_usart0(CHIP.unwrap().pm) };
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
        for &c in buf {
            uart.send_byte(regs_manager, c);
            while !uart.tx_ready(regs_manager) {}
        }
    }
}

/// Panic handler.
#[cfg(not(test))]
#[no_mangle]
#[panic_handler]
pub unsafe extern "C" fn panic_fmt(pi: &PanicInfo) -> ! {
    // turn off the non panic leds, just in case
    let led_green = sam4l::gpio::GPIOPin::new(sam4l::gpio::Pin::PA14);
    led_green.enable_output();
    led_green.set();
    let led_blue = sam4l::gpio::GPIOPin::new(sam4l::gpio::Pin::PA15);
    led_blue.enable_output();
    led_blue.set();

    let mut red_pin = sam4l::gpio::GPIOPin::new(sam4l::gpio::Pin::PA13);
    let led_red = &mut led::LedLow::new(&mut red_pin);
    let writer = &mut WRITER;
    debug::panic(
        &mut [led_red],
        writer,
        pi,
        &cortexm4::support::nop,
        &PROCESSES,
        &CHIP,
    )
}
