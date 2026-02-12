// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::panic::PanicInfo;
use kernel::debug;
use kernel::hil::gpio;
use kernel::hil::led;

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

    debug::panic::<_, sifive::uart::Uart, _, _>(
        &mut [led_red],
        sifive::uart::UartPanicWriterConfig {
            registers: arty_e21_chip::uart::UART0_BASE,
            clock_frequency: 32_000_000,
            params: kernel::hil::uart::Parameters {
                baud_rate: 115200,
                stop_bits: kernel::hil::uart::StopBits::One,
                parity: kernel::hil::uart::Parity::None,
                hw_flow_control: false,
                width: kernel::hil::uart::Width::Eight,
            },
        },
        pi,
        &rv32i::support::nop,
        crate::PANIC_RESOURCES.get(),
    )
}
