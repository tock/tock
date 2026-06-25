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
    // turn off the non panic leds, just in case
    let led_green = sam4l::gpio::GPIOPin::new(sam4l::gpio::Pin::PA14);
    led_green.enable_output();
    led_green.set();
    let led_blue = sam4l::gpio::GPIOPin::new(sam4l::gpio::Pin::PA15);
    led_blue.enable_output();
    led_blue.set();

    let red_pin = sam4l::gpio::GPIOPin::new(sam4l::gpio::Pin::PA13);
    let led_red = &mut led::LedLow::new(&red_pin);

    debug::panic::<_, USART, _, _>(
        &mut [led_red],
        UsartPanicWriterConfig {
            id: UsartId::Usart0,
        },
        pi,
        &cortexm4::support::nop,
        crate::PANIC_RESOURCES.get(),
    )
}
