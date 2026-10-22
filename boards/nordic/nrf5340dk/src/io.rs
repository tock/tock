// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use kernel::debug;
use kernel::hil::led;
use kernel::hil::uart;
use nrf5340::gpio::{GPIO_BASE_PORT0, GPIOPin, GPIOTE0_BASE, Pin};

#[cfg(not(test))]
#[panic_handler]
/// Panic handler
pub fn panic_fmt(pi: &core::panic::PanicInfo) -> ! {
    // LED1 on the nRF5340 DK.
    let led_kernel_pin = &GPIOPin::new(Pin::P0_28, GPIOTE0_BASE, GPIO_BASE_PORT0);
    let led = &mut led::LedLow::new(led_kernel_pin);

    // SAFETY: The kernel's UARTE0 driver never runs again after a panic, so
    // this is the only user of the UARTE0 DMA registers from here on.
    let registers =
        unsafe { nrf5340::uarte::UarteRegistersManager::new(nrf5340::chip::UARTE0_BASE) };

    debug::panic::<_, nrf5340::uarte::Uarte<'static>, _, _>(
        &mut [led],
        nrf5340::uarte::UartePanicWriterConfig {
            registers,
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
        &cortexm33::support::nop,
        crate::PANIC_RESOURCES.get(),
    )
}
