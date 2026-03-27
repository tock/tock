// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::fmt::Write;
use core::panic::PanicInfo;
use core::ptr::addr_of_mut;
use kernel::debug;
use kernel::debug::IoWrite;
use kernel::hil::led;
use kernel::hil::led::Led;

use crate::PANIC_RESOURCES;
use stm32u545::gpio::PinId;

/// Writer is used by kernel::debug to panic message to the serial port.
pub struct Writer {
    initialized: bool,
}

/// Global static for debug writer
pub static mut WRITER: Writer = Writer { initialized: false };

impl Writer {
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
        let uart_base = 0x46025000 as *mut u32; // LPUART1
        let isr = uart_base.wrapping_offset(0x1C / 4);
        let tdr = uart_base.wrapping_offset(0x28 / 4);

        for &c in buf {
            unsafe {
                while (*isr & (1 << 7)) == 0 {} // Wait for TXE
                *tdr = c as u32;
            }
        }
        buf.len()
    }
}

/// Panic handler.
#[panic_handler]
pub unsafe fn panic_fmt(info: &PanicInfo) -> ! {
    let rcc = stm32u545::rcc::Rcc::new();
    let clocks = stm32u545::clocks::Clocks::<stm32u545::chip_specs::Stm32u545Specs>::new(&rcc);
    let gpio_ports = stm32u545::gpio::GpioPorts::new(&clocks);
    let pin = stm32u545::gpio::Pin::new(PinId::PA05);
    pin.set_ports_ref(&gpio_ports);
    let mut led = led::LedHigh::new(&pin);
    led.init();

    let writer = &mut *addr_of_mut!(WRITER);

    debug::panic(
        &mut [&mut led],
        writer,
        info,
        &cortexm33::support::nop,
        PANIC_RESOURCES.as_ref().map(|r| *r),
    )
}
