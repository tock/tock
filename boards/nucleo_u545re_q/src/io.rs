// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright OxidOS Automotive 2026.

use core::panic::PanicInfo;

use kernel::debug;
use kernel::hil::uart;
use stm32u545::usart::{Usart, UsartPanicWriterConfig};

/// Panic handler.
#[panic_handler]
pub unsafe fn panic_fmt(info: &PanicInfo) -> ! {
    debug::panic_print::<Usart, crate::ChipHw, crate::ProcessPrinterInUse>(
        UsartPanicWriterConfig {
            registers: crate::PANIC_USART,
            params: uart::Parameters {
                baud_rate: 115200,
                stop_bits: uart::StopBits::One,
                parity: uart::Parity::None,
                hw_flow_control: false,
                width: uart::Width::Eight,
            },
        },
        info,
        &cortexm33::support::nop,
        crate::PANIC_RESOURCES.get(),
    );

    loop {}
}
