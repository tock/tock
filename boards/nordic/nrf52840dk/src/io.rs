// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::fmt::Write;
use kernel::debug::IoWrite;
use kernel::hil::uart;
use kernel::hil::uart::Configure;
use kernel::static_init;

use nrf52840::uart::{Uarte, UARTE0_BASE};

enum Writer {
    WriterUart(/* initialized */ bool),
    WriterRtt(&'static segger::rtt::SeggerRttMemory<'static>),
}

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

impl IoWrite for Writer {
    fn write(&mut self, buf: &[u8]) -> usize {
        match self {
            Writer::WriterUart(ref mut initialized) => {
                // Here, we create a second instance of the Uarte struct.
                // This is okay because we only call this during a panic, and
                // we will never actually process the interrupts
                let uart = Uarte::new(UARTE0_BASE);
                if !*initialized {
                    *initialized = true;
                    let _ = uart.configure(uart::Parameters {
                        baud_rate: 115200,
                        stop_bits: uart::StopBits::One,
                        parity: uart::Parity::None,
                        hw_flow_control: false,
                        width: uart::Width::Eight,
                    });
                }
                for &c in buf {
                    unsafe { uart.send_byte(c) }
                    while !uart.tx_ready() {}
                }
            }
            Writer::WriterRtt(rtt_memory) => rtt_memory.write_sync(buf),
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

    let writer = crate::RTT_BUFFER.get().map_or_else(
        || static_init!(Writer, Writer::WriterUart(false)),
        |buffer_cell| {
            buffer_cell.take().map_or_else(
                || static_init!(Writer, Writer::WriterUart(false)),
                |buffer| static_init!(Writer, Writer::WriterRtt(buffer)),
            )
        },
    );

    debug::panic(
        &mut [led],
        writer,
        pi,
        &cortexm4::support::nop,
        crate::PANIC_RESOURCES.get(),
    )
}
