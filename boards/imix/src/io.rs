// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::panic::PanicInfo;
use kernel::debug;
use kernel::hil::led;
use sam4l::usart::{UsartId, UsartPanicWriterConfig, USART};

/// Panic handler.
#[cfg(not(test))]
#[panic_handler]
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    let led_pin = sam4l::gpio::GPIOPin::new(sam4l::gpio::Pin::PC22);
    let led = &mut led::LedLow::new(&led_pin);

    debug::panic::<_, USART, _, _>(
        &mut [led],
        UsartPanicWriterConfig {
            id: UsartId::Usart3,
        },
        pi,
        &cortexm4::support::nop,
        crate::PANIC_RESOURCES.get(),
    )
}
