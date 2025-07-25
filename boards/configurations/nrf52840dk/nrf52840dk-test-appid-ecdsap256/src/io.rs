// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

use core::panic::PanicInfo;
use nrf52840::gpio::Pin;

#[cfg(not(test))]
#[panic_handler]
/// Panic handler
pub unsafe fn panic_fmt(_pi: &PanicInfo) -> ! {
    // The nRF52840DK LEDs (see back of board)
    let led_kernel_pin = &nrf52840::gpio::GPIOPin::new(Pin::P0_13);
    let led = &mut kernel::hil::led::LedLow::new(led_kernel_pin);
    kernel::debug::panic_blink_forever(&mut [led])
}
