// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::panic::PanicInfo;
use kernel::debug;

/// Panic handler.
#[cfg(not(test))]
#[panic_handler]
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    debug::panic_print::<esp32::uart::Uart, _, _>(
        esp32::uart::UartPanicWriterConfig {
            registers: esp32::uart::UART0_BASE,
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
    );

    loop {
        rv32i::support::nop();
    }
}

#[cfg(test)]
#[panic_handler]
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    debug::panic_print::<esp32::uart::Uart, _, _>(
        esp32::uart::UartPanicWriterConfig {
            registers: esp32::uart::UART0_BASE,
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
    );

    loop {
        rv32i::support::nop();
    }
}
