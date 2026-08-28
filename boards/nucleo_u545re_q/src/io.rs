// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright OxidOS Automotive 2026.

use core::panic::PanicInfo;
use kernel::debug;

/// Panic handler.
#[panic_handler]
pub unsafe fn panic_fmt(info: &PanicInfo) -> ! {
    let writer_config = stm32u545::usart::UsartPanicWriterConfig {
        base: stm32u545::usart::USART1_BASE,
    };

    debug::panic_print::<stm32u545::usart::Usart, crate::ChipHw, crate::ProcessPrinterInUse>(
        writer_config,
        info,
        &cortexm33::support::nop,
        crate::PANIC_RESOURCES.get(),
    );

    loop {}
}
