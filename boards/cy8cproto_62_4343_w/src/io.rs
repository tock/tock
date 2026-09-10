// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025 SRL.

use core::panic::PanicInfo;

use psoc62xa::gpio::GpioPin;
use psoc62xa::scb::{Scb, ScbPanicWriterConfig};

use kernel::debug;
use kernel::hil::led::LedHigh;
use kernel::hil::uart;

/// Panic handler for the CY8CPROTO-062-4343 board.
#[panic_handler]
pub unsafe fn panic_fmt(panic_info: &PanicInfo) -> ! {
    let led_kernel_pin = &GpioPin::new(psoc62xa::gpio::PsocPin::P13_7);
    let led = &mut LedHigh::new(led_kernel_pin);

    debug::panic::<_, Scb, _, _>(
        &mut [led],
        ScbPanicWriterConfig {
            params: uart::Parameters {
                baud_rate: 115200,
                stop_bits: uart::StopBits::One,
                parity: uart::Parity::None,
                hw_flow_control: false,
                width: uart::Width::Eight,
            },
        },
        panic_info,
        &cortexm0p::support::nop,
        crate::PANIC_RESOURCES.get(),
    )
}
