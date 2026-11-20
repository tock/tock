// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::panic::PanicInfo;

use kernel::debug;
use kernel::hil::led;
use kernel::hil::uart;
use kernel::utilities::StaticRef;

use stm32f429zi::chip_specs::Stm32f429Specs;
use stm32f429zi::clocks::phclk::{PCLK2, PeripheralClock, PeripheralClockType};
use stm32f429zi::dma::Dma2;
use stm32f429zi::gpio::PinId;
use stm32f429zi::usart::{USART1_BASE, Usart, UsartPanicWriterConfig, UsartRegisters};

/// The USART the panic handler writes to, which is the one the console uses.
const PANIC_USART: StaticRef<UsartRegisters> = USART1_BASE;

/// Panic handler.
#[panic_handler]
pub fn panic_fmt(info: &PanicInfo) -> ! {
    // User LD4 is connected to PG14
    // Have to reinitialize several peripherals because otherwise can't access them here.
    let rcc = stm32f429zi::rcc::Rcc::new();
    let clocks: stm32f429zi::clocks::Clocks<Stm32f429Specs> =
        stm32f429zi::clocks::Clocks::new(&rcc);
    let syscfg = stm32f429zi::syscfg::Syscfg::new(&clocks);
    let exti = stm32f429zi::exti::Exti::new(&syscfg);
    let pin = stm32f429zi::gpio::Pin::new(PinId::PG14, &exti);
    let gpio_ports = stm32f429zi::gpio::GpioPorts::new(&clocks, &exti);
    pin.set_ports_ref(&gpio_ports);
    let led = &mut led::LedHigh::new(&pin);

    debug::panic::<_, Usart<'_, Dma2<'_>>, _, _>(
        &mut [led],
        UsartPanicWriterConfig {
            registers: PANIC_USART,
            clock: PeripheralClock::new(PeripheralClockType::APB2(PCLK2::USART1), &clocks),
            params: uart::Parameters {
                baud_rate: 115200,
                stop_bits: uart::StopBits::One,
                parity: uart::Parity::None,
                hw_flow_control: false,
                width: uart::Width::Eight,
            },
        },
        info,
        &cortexm4::support::nop,
        crate::PANIC_RESOURCES.get(),
    )
}
