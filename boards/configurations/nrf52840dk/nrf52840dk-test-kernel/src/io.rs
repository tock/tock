// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

use core::fmt::Write;
use kernel::debug::IoWrite;
use kernel::hil::uart;
use kernel::hil::uart::Configure;

use nrf52840::uart::{Uarte, UARTE0_BASE};

struct Writer {
    initialized: bool,
}

impl Writer {
    fn new() -> Self {
        Self { initialized: false }
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
        // Here, we create a second instance of the Uarte struct.
        // This is okay because we only call this during a panic, and
        // we will never actually process the interrupts
        let uart = Uarte::new(UARTE0_BASE);
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
            unsafe {
                uart.send_byte(c);
            }
            while !uart.tx_ready() {}
        }

        buf.len()
    }
}

#[cfg(not(test))]
#[panic_handler]
/// Panic handler
pub unsafe fn panic_fmt(pi: &core::panic::PanicInfo) -> ! {
    use kernel::debug;
    use kernel::hil::led;
    use nrf52840::gpio::Pin;

    // The nRF52840DK LEDs (see back of board)
    let led_kernel_pin = &nrf52840::gpio::GPIOPin::new(Pin::P0_13);
    let led = &mut led::LedLow::new(led_kernel_pin);
    let mut writer = Writer::new();
    debug::panic_new(
        &mut [led],
        &mut writer,
        pi,
        &cortexm4::support::nop,
        crate::PANIC_RESOURCES.get(),
    )
}
