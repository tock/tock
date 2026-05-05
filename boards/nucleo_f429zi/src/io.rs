// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::panic::PanicInfo;

use kernel::debug;
use kernel::hil::led;

use stm32f429zi::chip_specs::Stm32f429Specs;
use stm32f429zi::gpio::PinId;

/// Panic handler.
#[panic_handler]
pub unsafe fn panic_fmt(info: &PanicInfo) -> ! {
    // User LD2 is connected to PB07
    let rcc = kernel::static_init!(stm32f429zi::rcc::Rcc, stm32f429zi::rcc::Rcc::new());
    let clocks = kernel::static_init!(
        stm32f429zi::clocks::Clocks<Stm32f429Specs>,
        stm32f429zi::clocks::Clocks::new(rcc)
    );
    let syscfg = stm32f429zi::syscfg::Syscfg::new(clocks);
    let exti = stm32f429zi::exti::Exti::new(&syscfg);
    let pin = stm32f429zi::gpio::Pin::new(PinId::PB07, &exti);
    let gpio_ports = stm32f429zi::gpio::GpioPorts::new(clocks, &exti);
    pin.set_ports_ref(&gpio_ports);
    let led = &mut led::LedHigh::new(&pin);

    debug::panic::<_, stm32f429zi::usart::Usart<'static, stm32f429zi::dma::Dma1<'static>>, _, _>(
        &mut [led],
        stm32f429zi::usart::UsartPanicWriterConfig {
            id: stm32f429zi::usart::UsartId::Usart3,
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
