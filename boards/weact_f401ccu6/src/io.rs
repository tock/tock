// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::panic::PanicInfo;

use kernel::debug;
use kernel::hil::led;

use stm32f401cc::chip_specs::Stm32f401Specs;
use stm32f401cc::gpio::PinId;

/// Panic handler.
#[panic_handler]
pub unsafe fn panic_fmt(info: &PanicInfo) -> ! {
    // On-board LED C13 is connected to PC13
    let rcc = kernel::static_init!(stm32f401cc::rcc::Rcc, stm32f401cc::rcc::Rcc::new());
    let clocks = kernel::static_init!(
        stm32f401cc::clocks::Clocks<Stm32f401Specs>,
        stm32f401cc::clocks::Clocks::new(rcc)
    );
    let syscfg = stm32f401cc::syscfg::Syscfg::new(clocks);
    let exti = stm32f401cc::exti::Exti::new(&syscfg);
    let pin = stm32f401cc::gpio::Pin::new(PinId::PC13, &exti);
    let gpio_ports = stm32f401cc::gpio::GpioPorts::new(clocks, &exti);
    pin.set_ports_ref(&gpio_ports);
    let led = &mut led::LedLow::new(&pin);

    debug::panic::<_, stm32f401cc::usart::Usart<'static, stm32f401cc::dma::Dma1<'static>>, _, _>(
        &mut [led],
        stm32f401cc::usart::UsartPanicWriterConfig {
            id: stm32f401cc::usart::UsartId::Usart2,
            clocks,
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
