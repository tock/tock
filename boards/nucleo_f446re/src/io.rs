// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::panic::PanicInfo;

use kernel::debug;
use kernel::hil::led;

use stm32f446re::chip_specs::Stm32f446Specs;
use stm32f446re::gpio::PinId;

/// Panic handler.
#[panic_handler]
pub unsafe fn panic_fmt(info: &PanicInfo) -> ! {
    // User LD2 is connected to PA05
    let rcc = kernel::static_init!(stm32f446re::rcc::Rcc, stm32f446re::rcc::Rcc::new());
    let clocks = kernel::static_init!(
        stm32f446re::clocks::Clocks<Stm32f446Specs>,
        stm32f446re::clocks::Clocks::new(rcc)
    );
    let syscfg = stm32f446re::syscfg::Syscfg::new(clocks);
    let exti = stm32f446re::exti::Exti::new(&syscfg);
    let pin = stm32f446re::gpio::Pin::new(PinId::PA05, &exti);
    let gpio_ports = stm32f446re::gpio::GpioPorts::new(clocks, &exti);
    pin.set_ports_ref(&gpio_ports);
    let led = &mut led::LedHigh::new(&pin);

    debug::panic::<_, stm32f446re::usart::Usart<'static, stm32f446re::dma::Dma1<'static>>, _, _>(
        &mut [led],
        stm32f446re::usart::UsartPanicWriterConfig {
            id: stm32f446re::usart::UsartId::Usart2,
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
