// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::panic::PanicInfo;

use kernel::hil::uart::{Parameters, Parity, StopBits, Width};

use nrf52840::gpio::Pin;
use nrf52840::uart::{UartPanicWriterConfig, Uarte};

const LED2_R_PIN: Pin = Pin::P0_13;

#[cfg(not(test))]
#[panic_handler]
/// Panic handler
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    use kernel::debug;
    use kernel::hil::led;

    let led_kernel_pin = &nrf52840::gpio::nrf52840_gpio_create_pin(LED2_R_PIN);
    let led = &mut led::LedLow::new(led_kernel_pin);

    debug::panic::<_, Uarte, _, _>(
        &mut [led],
        UartPanicWriterConfig {
            params: Parameters {
                baud_rate: 115200,
                stop_bits: StopBits::One,
                parity: Parity::None,
                hw_flow_control: false,
                width: Width::Eight,
            },
            txd: Pin::P0_06,
            rxd: Pin::P0_08,
            cts: None,
            rts: None,
        },
        pi,
        &cortexm4::support::nop,
        crate::PANIC_RESOURCES.get(),
    )
}
