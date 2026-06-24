// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Infineon Technologies AG 2026.

//! Board‑level I/O and panic infrastructure for the PSC3M5-EVK.

use core::panic::PanicInfo;
use psc3::gpio;
use psc3::gpio::GpioPin;

use kernel::debug;
use kernel::hil::led::LedHigh;

/// This function is called on panic, and it will attempt to print the panic message to the serial port.
/// It also blinks the LED to indicate a panic has occurred.
#[panic_handler]
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    let led_kernel_pin = &GpioPin::new(gpio::PsocPin::P8_5);
    let led = &mut LedHigh::new(led_kernel_pin);

    debug::panic::<_, psc3::scb::Scb, _, _>(
        &mut [led],
        psc3::scb::ScbPanicWriterConfig {
            params: kernel::hil::uart::Parameters {
                baud_rate: 115200,
                stop_bits: kernel::hil::uart::StopBits::One,
                parity: kernel::hil::uart::Parity::None,
                hw_flow_control: false,
                width: kernel::hil::uart::Width::Eight,
            },
        },
        pi,
        &cortexm33::support::nop,
        crate::PANIC_RESOURCES.get(),
    );
}
