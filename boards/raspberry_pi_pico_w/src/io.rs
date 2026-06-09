// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.
// Copyright OxidOS Automotive 2025.

use core::panic::PanicInfo;

use kernel::debug;
use kernel::hil::uart::{Parameters, Parity, StopBits, Width};

use rp2040::clocks::Clocks;
use rp2040::uart::{Uart, UartId, UartPanicWriterConfig};

/// Default panic handler for the Raspberry Pi Pico board.
///
/// We just use the standard default provided by the debug module in the kernel.
#[cfg(not(test))]
#[panic_handler]
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    let clocks = Clocks::new();
    debug::panic_print::<Uart, _, _>(
        UartPanicWriterConfig {
            id: UartId::Uart0,
            params: Parameters {
                baud_rate: 115200,
                width: Width::Eight,
                parity: Parity::None,
                stop_bits: StopBits::One,
                hw_flow_control: false,
            },
            clocks: &clocks,
        },
        pi,
        &cortexm0p::support::nop,
        raspberry_pi_pico::PANIC_RESOURCES.get(),
    );

    // Loop forever
    loop {}
}
