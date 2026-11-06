// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use kernel::debug;
use kernel::hil::led::LedLow;
use kernel::hil::uart;
use nrf5340_net::gpio::GPIOPin;

#[cfg(not(test))]
#[panic_handler]
/// Panic handler
pub fn panic_fmt(pi: &core::panic::PanicInfo) -> ! {
    // SAFETY: The kernel's UARTE0 driver never runs again after a panic, so
    // this is the only user of the UARTE0 DMA registers from here on.
    let registers =
        unsafe { nrf5340_net::uarte::UarteRegistersManager::new(nrf5340_net::chip::UARTE0_BASE) };

    // The DK's LEDs belong to the application core, so there is none to blink.
    debug::panic::<LedLow<'static, GPIOPin<'static>>, nrf5340_net::uarte::Uarte<'static>, _, _>(
        &mut [],
        nrf5340_net::uarte::UartePanicWriterConfig {
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
