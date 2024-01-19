// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::fmt::Write;
use core::panic::PanicInfo;

use cortexm4;

use kernel::debug;
use kernel::debug::IoWrite;
use kernel::hil::led;
use kernel::hil::uart;
use kernel::hil::uart::Configure;

use stm32f429zi;
use stm32f429zi::gpio::PinId;

use crate::CHIP;
use crate::PROCESSES;
use crate::PROCESS_PRINTER;

/// Writer is used by kernel::debug to panic message to the serial port.
pub struct Writer {
    initialized: bool,
}

/// Global static for debug writer
pub static mut WRITER: Writer = Writer { initialized: false };

impl Writer {
    /// Indicate that USART has already been initialized. Trying to double
    /// initialize USART1 causes stm32f429zi to go into in in-deterministic state.
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
    fn write(&mut self, buf: &[u8]) -> usize {
        let rcc = stm32f429zi::rcc::Rcc::new();
        let uart = stm32f429zi::usart::Usart::new_usart1(&rcc);

        if !self.initialized {
            self.initialized = true;

            let _ = uart.configure(uart::Parameters {
                baud_rate: 115200,
                stop_bits: uart::StopBits::One,
                parity: uart::Parity::None,
                hw_flow_control: false,
                width: uart::Width::Eight,
            });
        }

        for &c in buf {
            uart.send_byte(c);
        }

        buf.len()
    }
}

/// Panic handler.
#[no_mangle]
#[panic_handler]
pub unsafe fn panic_fmt(info: &PanicInfo) -> ! {
    // User LD4 is connected to PG14
    // Have to reinitialize several peripherals because otherwise can't access them here.
    let rcc = stm32f429zi::rcc::Rcc::new();
    let syscfg = stm32f429zi::syscfg::Syscfg::new(&rcc);
    let exti = stm32f429zi::exti::Exti::new(&syscfg);
    let pin = stm32f429zi::gpio::Pin::new(PinId::PG14, &exti);
    let gpio_ports = stm32f429zi::gpio::GpioPorts::new(&rcc, &exti);
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
        &PROCESS_PRINTER,
    )
}
