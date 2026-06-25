// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::panic::PanicInfo;

use kernel::debug;
use kernel::hil::led;

use imxrt10xx::gpio::PinId;

/// Panic handler.
#[panic_handler]
pub unsafe fn panic_fmt(info: &PanicInfo) -> ! {
    // User Led is connected to AdB0_09
    let pin = imxrt10xx::gpio::Pin::from_pin_id(PinId::AdB0_09);
    let led = &mut led::LedLow::new(&pin);

    let ccm = kernel::static_init!(imxrt10xx::ccm::Ccm, imxrt10xx::ccm::Ccm::new());

    debug::panic::<_, imxrt10xx::lpuart::Lpuart, _, _>(
        &mut [led],
        imxrt10xx::lpuart::LpuartPanicWriterConfig {
            ccm,
            id: imxrt10xx::lpuart::LpuartId::Lpuart1,
            params: kernel::hil::uart::Parameters {
                baud_rate: 115200,
                stop_bits: kernel::hil::uart::StopBits::One,
                parity: kernel::hil::uart::Parity::None,
                hw_flow_control: false,
                width: kernel::hil::uart::Width::Eight,
            },
        },
        info,
        &cortexm7::support::nop,
        crate::PANIC_RESOURCES.get(),
    )
}
