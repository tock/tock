// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::panic::PanicInfo;
use kernel::debug;
use kernel::hil::led;
use kernel::hil::uart::{Parameters, Parity, StopBits, Width};
use msp432::gpio::IntPinNr;
use msp432::uart::{Uart, UartId, UartPanicWriterConfig};
use msp432::wdt::Wdt;

/// Panic handler
#[panic_handler]
pub unsafe fn panic_fmt(info: &PanicInfo) -> ! {
    const LED1_PIN: IntPinNr = IntPinNr::P01_0;
    let gpio_pin = msp432::gpio::IntPin::new(LED1_PIN);
    let led = &mut led::LedHigh::new(&gpio_pin);

    let wdt = Wdt::new();
    wdt.disable();

    debug::panic::<_, Uart, _, _>(
        &mut [led],
        UartPanicWriterConfig {
            id: UartId::UcA0,
            params: Parameters {
                baud_rate: 115200,
                stop_bits: StopBits::One,
                parity: Parity::None,
                hw_flow_control: false,
                width: Width::Eight,
            },
        },
        info,
        &cortexm4::support::nop,
        crate::PANIC_RESOURCES.get(),
    )
}
