// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::panic::PanicInfo;
use kernel::debug;
use kernel::hil::led;
use kernel::hil::uart;
use nrf52840::gpio::Pin;
use nrf52840::uart::{UartPanicWriterConfig, Uarte};

const LED2_R_PIN: Pin = Pin::P0_13;

#[cfg(not(test))]
#[panic_handler]
/// Panic handler
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    // The nRF52840DK LEDs (see back of board)

    let led_kernel_pin = &nrf52840::gpio::nrf52840_gpio_create_pin(LED2_R_PIN);
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
            txd: crate::UART_TXD,
            rxd: crate::UART_RXD,
            cts: None,
            rts: None,
        },
        pi,
        &cortexm4::support::nop,
        crate::PANIC_RESOURCES.get(),
    )
}
