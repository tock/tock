// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use core::panic::PanicInfo;

use kernel::debug;
use nxp_s32g3::linflexd::LinFlexD;

/// Panic handler using the chip-owned synchronous LF0 writer.
#[panic_handler]
pub unsafe fn panic_fmt(info: &PanicInfo) -> ! {
    debug::panic_print::<LinFlexD, crate::ChipHw, crate::ProcessPrinterInUse>(
        (),
        info,
        &cortexm7::support::nop,
        crate::PANIC_RESOURCES.get(),
    );
    loop {}
}
