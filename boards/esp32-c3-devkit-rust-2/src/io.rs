// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use core::panic::PanicInfo;
use kernel::debug;
use kernel::hil::led::LedHigh;

/// Panic handler.
#[panic_handler]
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    let led_pin = esp32::gpio::GpioPin::new(
        esp32::gpio::GPIO_BASE,
        esp32::gpio::IOMUX_BASE,
        esp32::gpio::pins::pin10,
    );
    let led = &mut LedHigh::new(&led_pin);

    debug::panic::<_, esp32_c3::usb_serial_jtag::UsbSerialJtag, _, _>(
        &mut [led],
        esp32_c3::usb_serial_jtag::UsbSerialJtagPanicWriterConfig {
            registers: esp32_c3::chip::USB_SERIAL_JTAG_BASE,
        },
        pi,
        &rv32i::support::nop,
        crate::PANIC_RESOURCES.get(),
    )
}
