// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::fmt::Write;
use core::panic::PanicInfo;
use kernel::debug;
use kernel::debug::IoWrite;
use kernel::hil::led;
use kernel::hil::uart;
use kernel::hil::uart::Configure;
use nrf52840::gpio::Pin;
use nrf52840::uart::{Uarte, UARTE0_BASE};

use crate::CHIP;
use crate::PROCESSES;
use crate::PROCESS_PRINTER;

// Expand here with more writing methods as required (rtt/cdc etc...)
enum Writer {
    WriterUart(/* initialized */ bool),
}

static mut WRITER: Writer = Writer::WriterUart(false);

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
        }
        buf.len()
    }
}

const LED2_R_PIN: Pin = Pin::P0_13;

#[cfg(not(test))]
#[panic_handler]
/// Panic handler
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    // The nRF52840DK LEDs (see back of board)

    use core::ptr::{addr_of, addr_of_mut};
    let led_kernel_pin = &nrf52840::gpio::GPIOPin::new(LED2_R_PIN);
    let led = &mut led::LedLow::new(led_kernel_pin);
    let writer = &mut *addr_of_mut!(WRITER);
    debug::panic(
        &mut [led],
        writer,
        pi,
        &cortexm4::support::nop,
        &*addr_of!(PROCESSES),
        &*addr_of!(CHIP),
        &*addr_of!(PROCESS_PRINTER),
    )
}
