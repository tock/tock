// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use core::panic::PanicInfo;

#[panic_handler]
/// Panic handler for the STM32U545 board.
pub fn panic_fmt(_info: &PanicInfo) -> ! {
    loop {
        cortexm33::support::nop();
    }
}
