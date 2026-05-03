// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use kernel::debug;
use kernel::hil::led;
use kernel::hil::uart;
use kernel::utilities::cells::MapCell;
use nrf52840::gpio::Pin;

#[cfg(not(test))]
#[panic_handler]
/// Panic handler
pub unsafe fn panic_fmt(pi: &core::panic::PanicInfo) -> ! {
    // The nRF52840DK LEDs (see back of board)
    let led_kernel_pin = &nrf52840::gpio::GPIOPin::new(Pin::P0_13);
    let led = &mut led::LedLow::new(led_kernel_pin);

    if crate::USB_DEBUGGING {
        // Use the RTT output that needs to be setup in main.rs.

        crate::RTT_BUFFER.get().and_then(MapCell::take).map_or_else(
            || debug::panic_blink_forever(&mut [led]),
            |rtt| {
                debug::panic::<_, segger::rtt::SeggerRttMemory, _, _>(
                    &mut [led],
                    rtt,
                    pi,
                    &cortexm4::support::nop,
                    crate::PANIC_RESOURCES.get(),
                )
            },
        )
    } else {
        // Use the nRF52 UART for panic output.

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
}
