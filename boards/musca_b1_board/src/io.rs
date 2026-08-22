// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

//! Board‑level I/O and panic infrastructure for the Musca B1.

use core::panic::PanicInfo;
use kernel::debug;

/// This function is called on panic, and it will attempt to print the panic message to the serial port.
#[panic_handler]
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    debug::panic_print::<musca_b1::uart::Uart, _, _>(
        musca_b1::uart::UartPanicWriterConfig {
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
    loop {}
}
