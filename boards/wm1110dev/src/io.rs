// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2023.

use core::panic::PanicInfo;

use kernel::debug;
use kernel::hil::led;
use kernel::hil::uart;
use nrf52840::gpio::Pin;

/// Default panic handler for the microbit board.
///
/// We just use the standard default provided by the debug module in the kernel.
#[cfg(not(test))]
#[panic_handler]
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    // Red Led
    let led_red_pin = &nrf52840::gpio::GPIOPin::new(Pin::P0_14);
    let led = &mut led::LedHigh::new(led_red_pin);

    debug::panic::<_, nrf52840::uart::Uarte, _, _>(
        &mut [led],
        nrf52840::uart::UartPanicWriterConfig {
            params: uart::Parameters {
                baud_rate: 115200,
                stop_bits: uart::StopBits::One,
                parity: uart::Parity::None,
                hw_flow_control: false,
                width: uart::Width::Eight,
            },
            txd: crate::UART_TX_PIN,
            rxd: crate::UART_RX_PIN,
            cts: None,
            rts: None,
        },
        pi,
        &cortexm4::support::nop,
        crate::PANIC_RESOURCES.get(),
    )
}
