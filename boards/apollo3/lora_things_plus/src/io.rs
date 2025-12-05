// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::panic::PanicInfo;

use kernel::debug;
use kernel::hil::led;
use kernel::utilities::io_write::IoWrite;

/// Panic handler.
#[panic_handler]
pub unsafe fn panic_fmt(info: &PanicInfo) -> ! {
    // just create a new pin reference here instead of using global
    let led_pin = &mut apollo3::gpio::GpioPin::new(
        kernel::utilities::StaticRef::new(
            apollo3::gpio::GPIO_BASE_RAW as *const apollo3::gpio::GpioRegisters,
        ),
        apollo3::gpio::Pin::Pin26,
    );
    let led = &mut led::LedLow::new(led_pin);

    debug::panic::<_, apollo3::uart::Uart, _, _>(
        &mut [led],
        apollo3::uart::UartPanicWriterConfig {
            params: kernel::hil::uart::Parameters {
                baud_rate: 115200,
                stop_bits: kernel::hil::uart::StopBits::One,
                parity: kernel::hil::uart::Parity::None,
                hw_flow_control: false,
                width: kernel::hil::uart::Width::Eight,
            },
            tx_pin_index: 48,
            rx_pin_index: 49,
        },
        info,
        &cortexm4::support::nop,
        crate::PANIC_RESOURCES.get(),
    )
}
