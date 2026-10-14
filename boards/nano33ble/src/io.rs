// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::panic::PanicInfo;

use kernel::debug;
use kernel::hil::led;
use nrf52840::gpio::Pin;

/// Default panic handler for the Nano 33 Board.
///
/// We just use the standard default provided by the debug module in the kernel.
#[cfg(not(test))]
#[panic_handler]
pub fn panic_fmt(pi: &PanicInfo) -> ! {
    let led_kernel_pin = &nrf52840::gpio::nrf52840_gpio_create_pin(Pin::P0_13);
    let led = &mut led::LedLow::new(led_kernel_pin);

    match super::CDC_REF_FOR_PANIC.get() {
        Some(data) => {
            debug::panic::<_, nrf52_components::usb_panic_writer::CdcPanicWriterData, _, _>(
                &mut [led],
                data,
                pi,
                &cortexm4::support::nop,
                crate::PANIC_RESOURCES.get(),
            )
        }
        None => debug::panic_blink_forever(&mut [led]),
    }
}
