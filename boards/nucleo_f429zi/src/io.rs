use core::fmt::Write;
use core::panic::PanicInfo;

use cortexm4;

use kernel::debug;
use kernel::hil::led;
use kernel::hil::uart::{self, UART};

use stm32f4xx;
use stm32f4xx::gpio::PinId;

use crate::PROCESSES;

/// Writer is used by kernel::debug to panic message to the serial port.
pub struct Writer {
    initialized: bool,
}

/// Global static for debug writer
pub static mut WRITER: Writer = Writer { initialized: false };

impl Writer {
    /// Indicate that USART has already been initialized. Trying to double
    /// initialize USART3 causes STM32F429ZI to go into in in-deterministic state.
    pub fn set_initialized(&mut self) {
        self.initialized = true;
    }
}

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        let uart = unsafe { &mut stm32f4xx::usart::USART3 };

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
        }

        Ok(())
    }
}

/// Panic handler.
#[no_mangle]
#[panic_handler]
pub unsafe extern "C" fn panic_fmt(info: &PanicInfo) -> ! {
    // User LD2 is connected to PB07
    PinId::PB07.get_pin_mut().as_mut().map(|pb7| {
        let led = &mut led::LedHigh::new(pb7);
        let writer = &mut WRITER;

        debug::panic(
            &mut [led],
            writer,
            info,
            &cortexm4::support::nop,
            &PROCESSES,
        )
    });

    loop {}
}
