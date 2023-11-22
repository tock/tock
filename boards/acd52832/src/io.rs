// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::panic::PanicInfo;

use kernel::debug;
use kernel::hil::led;
use nrf52832::gpio::Pin;

/// Panic.
#[cfg(not(test))]
#[no_mangle]
#[panic_handler]
pub unsafe fn panic_fmt(_pi: &PanicInfo) -> ! {
    let led_kernel_pin = &nrf52832::gpio::GPIOPin::new(Pin::P0_22);
    let led = &mut led::LedLow::new(led_kernel_pin);
    debug::panic_blink_forever(&mut [led])
}
