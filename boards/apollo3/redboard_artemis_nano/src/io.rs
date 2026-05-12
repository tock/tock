// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::panic::PanicInfo;

use kernel::debug;
use kernel::hil::led;
use kernel::hil::uart::{Parameters, Parity, StopBits, Width};

use apollo3::uart::{Uart, UartPanicWriterConfig, UART0_BASE};

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

    debug::panic::<_, Uart, _, _>(
        &mut [led],
        UartPanicWriterConfig {
            registers: UART0_BASE,
            params: Parameters {
                baud_rate: 115200,
                width: Width::Eight,
                stop_bits: StopBits::One,
                parity: Parity::None,
                hw_flow_control: false,
            },
        },
        info,
        &cortexm4::support::nop,
        crate::PANIC_RESOURCES.get(),
    )
}
