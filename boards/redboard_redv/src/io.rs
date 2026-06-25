// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::panic::PanicInfo;
use kernel::debug;
use kernel::hil::gpio;
use kernel::hil::led;
use kernel::hil::uart::{Parameters, Parity, StopBits, Width};

use sifive::uart::{Uart, UartPanicWriterConfig};

/// Panic handler.
#[cfg(not(test))]
#[panic_handler]
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    // turn off the non panic leds, just in case

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

    debug::panic::<_, Uart, _, _>(
        &mut [led_red],
        UartPanicWriterConfig {
            registers: e310_g002::uart::UART0_BASE,
            clock_frequency: 16_000_000,
            params: Parameters {
                baud_rate: 115200,
                width: Width::Eight,
                stop_bits: StopBits::One,
                parity: Parity::None,
                hw_flow_control: false,
            },
        },
        pi,
        &rv32i::support::nop,
        crate::PANIC_RESOURCES.get(),
    )
}
