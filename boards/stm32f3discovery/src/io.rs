// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::panic::PanicInfo;

use kernel::debug;
use kernel::hil::led;

use stm32f303xc::gpio::PinId;

/// Panic handler.
#[panic_handler]
pub unsafe fn panic_fmt(info: &PanicInfo) -> ! {
    // User LD3 is connected to PE09
    let rcc = kernel::static_init!(stm32f303xc::rcc::Rcc, stm32f303xc::rcc::Rcc::new());
    let syscfg = stm32f303xc::syscfg::Syscfg::new(rcc);
    let exti = stm32f303xc::exti::Exti::new(&syscfg);
    let pin = stm32f303xc::gpio::Pin::new(PinId::PE09, &exti);
    let gpio_ports = stm32f303xc::gpio::GpioPorts::new(rcc, &exti);
    pin.set_ports_ref(&gpio_ports);
    let led = &mut led::LedHigh::new(&pin);

    debug::panic::<_, stm32f303xc::usart::Usart, _, _>(
        &mut [led],
        stm32f303xc::usart::UsartPanicWriterConfig {
            id: stm32f303xc::usart::UsartId::Usart1,
            rcc,
            params: kernel::hil::uart::Parameters {
                baud_rate: 115200,
                stop_bits: kernel::hil::uart::StopBits::One,
                parity: kernel::hil::uart::Parity::None,
                hw_flow_control: false,
                width: kernel::hil::uart::Width::Eight,
            },
        },
        info,
        &cortexm4::support::nop,
        crate::PANIC_RESOURCES.get(),
    )
}
