// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::fmt::Write;
use core::panic::PanicInfo;
use kernel::debug;
use kernel::hil::led;
use kernel::hil::uart;
use kernel::hil::uart::Configure;
use kernel::utilities::io_write::IoWrite;
use nrf52840::gpio::Pin;
use nrf52840::uart::Uarte;

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
                let uart = Uarte::new(
                    crate::UARTE0_REGISTERS_MANAGER
                        .get()
                        .copied()
                        .expect("UARTE0_REGISTERS_MANAGER not bound to this thread"),
                );
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

    use core::ptr::addr_of_mut;
    let led_kernel_pin = &nrf52840::gpio::nrf52840_gpio_create_pin(LED2_R_PIN);
    let led = &mut led::LedLow::new(led_kernel_pin);
    let writer = &mut *addr_of_mut!(WRITER);
    debug::panic_old(
        &mut [led],
        writer,
        pi,
        &cortexm4::support::nop,
        crate::PANIC_RESOURCES.get(),
    )
}
