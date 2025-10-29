// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

use core::fmt::Write;
use kernel::debug::IoWrite;
use kernel::hil::uart;
use kernel::hil::uart::Configure;

use nrf52840::uart::{Uarte, UARTE0_BASE};

enum Writer {
    WriterUart(/* initialized */ bool),
    WriterRtt(&'static segger::rtt::SeggerRttMemory<'static>),
}

static mut WRITER: Writer = Writer::WriterUart(false);

/// Set the RTT memory buffer used to output panic messages.
pub unsafe fn set_rtt_memory(rtt_memory: &'static segger::rtt::SeggerRttMemory<'static>) {
    WRITER = Writer::WriterRtt(rtt_memory);
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
                    unsafe {
                        uart.send_byte(c);
                    }
                    while !uart.tx_ready() {}
                }
            }
            Writer::WriterRtt(rtt_memory) => {
                rtt_memory.write_sync(buf);
            }
        }
        buf.len()
    }
}

#[cfg(not(test))]
#[panic_handler]
/// Panic handler
pub unsafe fn panic_fmt(pi: &core::panic::PanicInfo) -> ! {
    use core::ptr::{addr_of, addr_of_mut};
    use kernel::debug;
    use kernel::hil::led;
    use nrf52840::gpio::Pin;

    use crate::CHIP;
    use crate::PROCESSES;
    use crate::PROCESS_PRINTER;

    // The nRF52840DK LEDs (see back of board)
    let led_kernel_pin = &nrf52840::gpio::GPIOPin::new(
        Pin::P0_13,
        nrf52840::gpio::GPIOTE_BASE,
        nrf52840::gpio::GPIO_BASE_PORT0,
    );
    let led = &mut led::LedLow::new(led_kernel_pin);
    let writer = &mut *addr_of_mut!(WRITER);
    debug::panic(
        &mut [led],
        writer,
        pi,
        &cortexm4::support::nop,
        PROCESSES.unwrap().as_slice(),
        &*addr_of!(CHIP),
        &*addr_of!(PROCESS_PRINTER),
    )
}
