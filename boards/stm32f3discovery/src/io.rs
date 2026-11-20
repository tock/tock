// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::panic::PanicInfo;

use kernel::debug;
use kernel::hil::led;
use kernel::utilities::StaticRef;

use stm32f303xc::gpio::PinId;
use stm32f303xc::rcc::{PCLK2, PeripheralClock, PeripheralClockType};
use stm32f303xc::usart::{USART1_BASE, Usart, UsartPanicWriterConfig, UsartRegisters};

/// The USART the panic handler writes to, which is the one the console uses.
const PANIC_USART: StaticRef<UsartRegisters> = USART1_BASE;

/// Panic handler.
#[panic_handler]
pub fn panic_fmt(info: &PanicInfo) -> ! {
    // User LD3 is connected to PE09
    // Have to reinitialize several peripherals because otherwise can't access them here.
    let rcc = stm32f303xc::rcc::Rcc::new();
    let syscfg = stm32f303xc::syscfg::Syscfg::new(&rcc);
    let exti = stm32f303xc::exti::Exti::new(&syscfg);
    let pin = stm32f303xc::gpio::Pin::new(PinId::PE09, &exti);
    let gpio_ports = stm32f303xc::gpio::GpioPorts::new(&rcc, &exti);
    pin.set_ports_ref(&gpio_ports);
    let led = &mut led::LedHigh::new(&pin);

    // No `uart::Parameters` to pass in here: this chip's USART driver only
    // ever supports one configuration (115200 8N1, no hardware flow
    // control), so there is nothing for the panic handler to configure
    // differently.
    debug::panic::<_, Usart<'_>, _, _>(
        &mut [led],
        UsartPanicWriterConfig {
            registers: PANIC_USART,
            clock: PeripheralClock::new(PeripheralClockType::APB2(PCLK2::USART1), &rcc),
        },
        info,
        &cortexm4::support::nop,
        crate::PANIC_RESOURCES.get(),
    )
}
