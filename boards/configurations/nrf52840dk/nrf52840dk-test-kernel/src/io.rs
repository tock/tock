// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

use kernel::hil::uart;
use nrf52840::uart::{UartPanicWriterConfig, Uarte};

#[cfg(not(test))]
#[panic_handler]
/// Panic handler
pub unsafe fn panic_fmt(pi: &core::panic::PanicInfo) -> ! {
    use kernel::debug;
    use kernel::hil::led;
    use nrf52840::gpio::Pin;

    // The nRF52840DK LEDs (see back of board)
    let led_kernel_pin = &nrf52840::gpio::nrf52840_gpio_create_pin(Pin::P0_13);
    let led = &mut led::LedLow::new(led_kernel_pin);
    debug::panic::<_, Uarte, _, _>(
        &mut [led],
        UartPanicWriterConfig {
            params: uart::Parameters {
                baud_rate: 115200,
                stop_bits: uart::StopBits::One,
                parity: uart::Parity::None,
                hw_flow_control: false,
                width: uart::Width::Eight,
            },
            txd: Pin::P0_06,
            rxd: Pin::P0_08,
            cts: Some(Pin::P0_07),
            rts: Some(Pin::P0_05),
        },
        pi,
        &cortexm4::support::nop,
        crate::PANIC_RESOURCES.get(),
    )
}
