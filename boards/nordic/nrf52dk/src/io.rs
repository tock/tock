// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::panic::PanicInfo;
use kernel::debug;
use kernel::hil::led;
use kernel::hil::uart;
use nrf52832::gpio::Pin;

#[cfg(not(test))]
#[panic_handler]
/// Panic handler
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    // The nRF52 DK LEDs (see back of board)

    let led_kernel_pin = &nrf52832::gpio::GPIOPin::new(Pin::P0_17);
    let led = &mut led::LedLow::new(led_kernel_pin);

    debug::panic_new::<nrf52832::uart::Uarte, _, _, _>(
        &mut [led],
        nrf52832::uart::UartPanicWriterConfig {
            params: uart::Parameters {
                baud_rate: 115200,
                stop_bits: uart::StopBits::One,
                parity: uart::Parity::None,
                hw_flow_control: false,
                width: uart::Width::Eight,
            },
            txd: crate::UART_TXD,
            rxd: crate::UART_RXD,
            cts: crate::UART_CTS,
            rts: crate::UART_CTS,
        },
        pi,
        &cortexm4::support::nop,
        crate::PANIC_RESOURCES.get(),
    )
}
