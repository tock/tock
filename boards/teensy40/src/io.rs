// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use kernel::debug;
use kernel::hil::{led, uart::Parameters, uart::Parity, uart::StopBits, uart::Width};

use crate::imxrt1060::gpio::PinId;
use crate::imxrt1060::lpuart::{Lpuart, LpuartId, LpuartPanicWriterConfig};

#[panic_handler]
unsafe fn panic_handler(panic_info: &core::panic::PanicInfo) -> ! {
    let ccm = kernel::static_init!(crate::imxrt1060::ccm::Ccm, crate::imxrt1060::ccm::Ccm::new());
    let pin = crate::imxrt1060::gpio::Pin::from_pin_id(PinId::B0_03);
    let led = &mut led::LedHigh::new(&pin);

    debug::panic::<_, Lpuart, _, _>(
        &mut [led],
        LpuartPanicWriterConfig {
            ccm,
            id: LpuartId::Lpuart2,
            params: Parameters {
                baud_rate: 115_200,
                stop_bits: StopBits::One,
                parity: Parity::None,
                hw_flow_control: false,
                width: Width::Eight,
            },
        },
        panic_info,
        &cortexm7::support::nop,
        crate::PANIC_RESOURCES.get(),
    )
}
