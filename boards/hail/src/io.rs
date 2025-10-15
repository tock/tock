// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::fmt::Write;
use core::panic::PanicInfo;
use core::str;
use kernel::debug;
use kernel::debug::IoWrite;
use kernel::hil::led;
use kernel::hil::uart::{self, Configure};

struct Writer {
    initialized: bool,
    uart: sam4l::usart::USART<'static>,
}

impl Writer {
    fn new(chip: &'static crate::Chip) -> Self {
        let uart = sam4l::usart::USART::new_usart0(chip.pm);
        Self {
            initialized: false,
            uart,
        }
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
        // Here, we create a second instance of the USART0 struct.
        // This is okay because we only call this during a panic, and
        // we will never actually process the interrupts
        // let uart = unsafe { sam4l::usart::USART::new_usart0(CHIP.unwrap().pm) };
        let uart = &self.uart;
        let regs_manager = &sam4l::usart::USARTRegManager::panic_new(&uart);
        if !self.initialized {
            self.initialized = true;
            let _ = uart.configure(uart::Parameters {
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
        buf.len()
    }
}

/// Panic handler.
#[cfg(not(test))]
#[panic_handler]
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    // turn off the non panic leds, just in case
    let led_green = sam4l::gpio::GPIOPin::new(sam4l::gpio::Pin::PA14);
    led_green.enable_output();
    led_green.set();
    let led_blue = sam4l::gpio::GPIOPin::new(sam4l::gpio::Pin::PA15);
    led_blue.enable_output();
    led_blue.set();

    let red_pin = sam4l::gpio::GPIOPin::new(sam4l::gpio::Pin::PA13);
    let led_red = &mut led::LedLow::new(&red_pin);

    crate::PANIC_RESOURCES.with(|resources| {
        resources.chip.take().map_or_else(
            || debug::panic_blink_forever(&mut [led_red]),
            |c| {
                let writer = kernel::static_init!(Writer, Writer::new(c));
                resources.set_chip(c);

                debug::panic(
                    &mut [led_red],
                    writer,
                    pi,
                    &cortexm4::support::nop,
                    resources,
                )
            },
        )
    })
}
