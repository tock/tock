use core::fmt::Write;
use core::panic::PanicInfo;

use kernel::debug;
use kernel::debug::IoWrite;
use kernel::hil::led;
use kernel::hil::uart;
use kernel::hil::uart::Configure;

use stm32f303xc;
use stm32f303xc::gpio::PinId;

use crate::CHIP;
use crate::PROCESSES;

/// Writer is used by kernel::debug to panic message to the serial port.
pub struct Writer {
    initialized: bool,
}

/// Global static for debug writer
pub static mut WRITER: Writer = Writer { initialized: false };

impl Writer {
    /// Indicate that USART has already been initialized. Trying to double
    /// initialize USART2 causes STM32F446RE to go into in in-deterministic state.
    pub fn set_initialized(&mut self) {
        self.initialized = true;
    }
}

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

impl IoWrite for Writer {
    fn write(&mut self, buf: &[u8]) {
        let rcc = stm32f303xc::rcc::Rcc::new();
        let uart = stm32f303xc::usart::Usart::new_usart1(&rcc);

        if !self.initialized {
            self.initialized = true;

            uart.configure(uart::Parameters {
                baud_rate: 11500,
                stop_bits: uart::StopBits::One,
                parity: uart::Parity::None,
                hw_flow_control: false,
                width: uart::Width::Eight,
            });
        }

        for &c in buf {
            uart.send_byte(c);
        }
    }
}

/// Panic handler.
#[no_mangle]
#[panic_handler]
pub unsafe extern "C" fn panic_fmt(info: &PanicInfo) -> ! {
    // User LD3 is connected to PE09
    // Have to reinitialize several peripherals because otherwise can't access them here.
    let rcc = stm32f303xc::rcc::Rcc::new();
    let syscfg = stm32f303xc::syscfg::Syscfg::new(&rcc);
    let exti = stm32f303xc::exti::Exti::new(&syscfg);
    let pin = stm32f303xc::gpio::Pin::new(PinId::PE09, &exti);
    let gpio_ports = stm32f303xc::gpio::GpioPorts::new(&rcc, &exti);
    pin.set_ports_ref(&gpio_ports);
    let led = &mut led::LedHigh::new(&pin);
    let writer = &mut WRITER;

    debug::panic(
        &mut [led],
        writer,
        info,
        &cortexm4::support::nop,
        &PROCESSES,
        &CHIP,
    )
}
