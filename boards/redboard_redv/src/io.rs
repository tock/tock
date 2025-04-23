// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::fmt::Write;
use core::panic::PanicInfo;
use core::str;
use kernel::debug;
use kernel::debug::IoWrite;
use kernel::hil::gpio;
use kernel::hil::led;

use crate::CHIP;
use crate::PROCESSES;
use crate::PROCESS_PRINTER;

struct Writer {}

static mut WRITER: Writer = Writer {};

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

impl IoWrite for Writer {
    fn write(&mut self, buf: &[u8]) -> usize {
        let uart = sifive::uart::Uart::new(e310_g002::uart::UART0_BASE, 16_000_000);
        uart.transmit_sync(buf);
        buf.len()
    }
}

/// Panic handler.
#[cfg(not(test))]
#[panic_handler]
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    // turn off the non panic leds, just in case

    use core::ptr::{addr_of, addr_of_mut};
    let led_green = sifive::gpio::GpioPin::new(
        e310_g002::gpio::GPIO0_BASE,
        sifive::gpio::pins::pin19,
        sifive::gpio::pins::pin19::SET,
        sifive::gpio::pins::pin19::CLEAR,
    );
    gpio::Configure::make_output(&led_green);
    gpio::Output::set(&led_green);

    let led_blue = sifive::gpio::GpioPin::new(
        e310_g002::gpio::GPIO0_BASE,
        sifive::gpio::pins::pin21,
        sifive::gpio::pins::pin21::SET,
        sifive::gpio::pins::pin21::CLEAR,
    );
    gpio::Configure::make_output(&led_blue);
    gpio::Output::set(&led_blue);

    let led_red_pin = sifive::gpio::GpioPin::new(
        e310_g002::gpio::GPIO0_BASE,
        sifive::gpio::pins::pin22,
        sifive::gpio::pins::pin22::SET,
        sifive::gpio::pins::pin22::CLEAR,
    );
    let led_red = &mut led::LedLow::new(&led_red_pin);
    let writer = &mut *addr_of_mut!(WRITER);

    debug::panic(
        &mut [led_red],
        writer,
        pi,
        &rv32i::support::nop,
        &*addr_of!(PROCESSES),
        &*addr_of!(CHIP),
        &*addr_of!(PROCESS_PRINTER),
    )
}
