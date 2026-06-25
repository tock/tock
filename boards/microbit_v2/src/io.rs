// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::panic::PanicInfo;

use kernel::debug;
use kernel::hil::led;
use kernel::hil::uart::{Parameters, Parity, StopBits, Width};
use nrf52833::gpio::Pin;
use nrf52833::uart::{UartPanicWriterConfig, Uarte};

/// Default panic handler for the microbit board.
///
/// We just use the standard default provided by the debug module in the kernel.
#[cfg(not(test))]
#[panic_handler]
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    // MicroBit v2 has a microphone LED, use it for panic
    let led_kernel_pin = &nrf52833::gpio::nrf52833_gpio_create_pin(Pin::P0_20);
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
            rxd: Pin::P1_08,
            cts: None,
            rts: None,
        },
        pi,
        &cortexm4::support::nop,
        crate::PANIC_RESOURCES.get(),
    )
}
