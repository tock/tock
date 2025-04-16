// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::fmt::Write;
use core::panic::PanicInfo;
use core::ptr::addr_of;
use core::ptr::addr_of_mut;

use crate::CHIP;
use crate::PROCESSES;
use crate::PROCESS_PRINTER;
use kernel::debug;
use kernel::debug::IoWrite;
use kernel::hil::led;

/// Writer is used by kernel::debug to panic message to the serial port.
pub struct Writer {}

/// Global static for debug writer
pub static mut WRITER: Writer = Writer {};

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

impl IoWrite for Writer {
    fn write(&mut self, buf: &[u8]) -> usize {
        let uart = apollo3::uart::Uart::new_uart_0(); // Aliases memory for uart0. Okay bc we are panicking.
        uart.transmit_sync(buf);
        buf.len()
    }
}

/// Panic handler.
#[panic_handler]
pub unsafe fn panic_fmt(info: &PanicInfo) -> ! {
    // just create a new pin reference here instead of using global
    let led_pin = &mut apollo3::gpio::GpioPin::new(
        kernel::utilities::StaticRef::new(
            apollo3::gpio::GPIO_BASE_RAW as *const apollo3::gpio::GpioRegisters,
        ),
        apollo3::gpio::Pin::Pin19,
    );
    let led = &mut led::LedLow::new(led_pin);
    let writer = &mut *addr_of_mut!(WRITER);

    debug::panic(
        &mut [led],
        writer,
        info,
        &cortexm4::support::nop,
        &*addr_of!(PROCESSES),
        &*addr_of!(CHIP),
        &*addr_of!(PROCESS_PRINTER),
    )
}
