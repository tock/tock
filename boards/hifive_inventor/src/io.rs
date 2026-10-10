// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::panic::PanicInfo;
use kernel::debug;
use kernel::hil::led;
use kernel::hil::uart::{Parameters, Parity, StopBits, Width};

use sifive::uart::{Uart, UartPanicWriterConfig};

/// Panic handler.
#[cfg(not(test))]
#[panic_handler]
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    let led = sifive::gpio::GpioPin::new(
        e310_g003::gpio::GPIO0_BASE,
        sifive::gpio::pins::pin22,
        sifive::gpio::pins::pin22::SET,
        sifive::gpio::pins::pin22::CLEAR,
    );
    let led = &mut led::LedLow::new(&led);

    debug::panic::<_, Uart, _, _>(
        &mut [led],
        UartPanicWriterConfig {
            registers: e310_g003::uart::UART0_BASE,
            clock_frequency: 16_000_000,
            params: Parameters {
                baud_rate: 115200,
                width: Width::Eight,
                stop_bits: StopBits::One,
                parity: Parity::None,
                hw_flow_control: false,
            },
        },
        pi,
        &rv32i::support::nop,
        crate::PANIC_RESOURCES.get(),
    )
}
