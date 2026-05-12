// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

use core::panic::PanicInfo;

use kernel::debug;
use kernel::hil::led::LedHigh;
use kernel::hil::uart::{Parameters, Parity, StopBits, Width};

use rp2350::gpio::{RPGpio, RPGpioPin};
use rp2350::uart::{Uart, UartId, UartPanicWriterConfig};

/// Default panic handler for the Raspberry Pi Pico 2 board.
///
/// We just use the standard default provided by the debug module in the kernel.
#[cfg(not(test))]
#[panic_handler]
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    // LED is connected to GPIO 25
    let led_kernel_pin = &RPGpioPin::new(RPGpio::GPIO25);
    let led = &mut LedHigh::new(led_kernel_pin);

    debug::panic::<_, Uart, _, _>(
        &mut [led],
        UartPanicWriterConfig {
            id: UartId::Uart0,
            params: Parameters {
                baud_rate: 115200,
                width: Width::Eight,
                parity: Parity::None,
                stop_bits: StopBits::One,
                hw_flow_control: false,
            },
        },
        pi,
        &cortexm33::support::nop,
        crate::PANIC_RESOURCES.get(),
    )
}
