// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::fmt::Write;
use core::panic::PanicInfo;
use core::ptr::addr_of;
use core::ptr::addr_of_mut;
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
        sifive::uart::Uart::new(arty_e21_chip::uart::UART0_BASE, 32_000_000).transmit_sync(buf);
        buf.len()
    }
}

/// Panic handler.
#[cfg(not(test))]
#[panic_handler]
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    // turn off the non panic leds, just in case
    let led_green = &sifive::gpio::GpioPin::new(
        arty_e21_chip::gpio::GPIO0_BASE,
        sifive::gpio::pins::pin1,
        sifive::gpio::pins::pin1::SET,
        sifive::gpio::pins::pin1::CLEAR,
    );
    gpio::Configure::make_output(led_green);
    gpio::Output::clear(led_green);

    let led_blue = &sifive::gpio::GpioPin::new(
        arty_e21_chip::gpio::GPIO0_BASE,
        sifive::gpio::pins::pin0,
        sifive::gpio::pins::pin0::SET,
        sifive::gpio::pins::pin0::CLEAR,
    );
    gpio::Configure::make_output(led_blue);
    gpio::Output::clear(led_blue);

    let led_red_pin = &mut sifive::gpio::GpioPin::new(
        arty_e21_chip::gpio::GPIO0_BASE,
        sifive::gpio::pins::pin2,
        sifive::gpio::pins::pin2::SET,
        sifive::gpio::pins::pin2::CLEAR,
    );

    let led_red = &mut led::LedHigh::new(led_red_pin);
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
