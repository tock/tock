// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use crate::CHIP;
use crate::PROCESSES;
use crate::PROCESS_PRINTER;

use core::fmt::Write;
use core::panic::PanicInfo;
use core::ptr::addr_of;
use core::ptr::addr_of_mut;
use kernel::debug;
use kernel::debug::IoWrite;
use kernel::hil::led;
use msp432::gpio::IntPinNr;
use msp432::wdt::Wdt;

/// Uart is used by kernel::debug to panic message to the serial port.
pub struct Uart {}

/// Global static for debug writer
pub static mut UART: Uart = Uart {};

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

impl IoWrite for Uart {
    fn write(&mut self, buf: &[u8]) -> usize {
        let uart0 = msp432::uart::Uart::new(msp432::usci::USCI_A0_BASE, 0, 1, 1, 1);
        uart0.transmit_sync(buf);
        buf.len()
    }
}

/// Panic handler
#[panic_handler]
pub unsafe fn panic_fmt(info: &PanicInfo) -> ! {
    const LED1_PIN: IntPinNr = IntPinNr::P01_0;
    let gpio_pin = msp432::gpio::IntPin::new(LED1_PIN);
    let led = &mut led::LedHigh::new(&gpio_pin);
    let writer = &mut *addr_of_mut!(UART);
    let wdt = Wdt::new();

    wdt.disable();
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
