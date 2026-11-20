// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

use core::panic::PanicInfo;

use kernel::debug;
use kernel::hil::led;
use kernel::utilities::StaticRef;
use stm32wle5jc::chip_specs::Stm32wle5jcSpecs;
use stm32wle5jc::clocks::phclk::{PCLK2, PeripheralClock, PeripheralClockType};
use stm32wle5jc::gpio::PinId;
use stm32wle5jc::usart::{USART1_BASE, Usart, UsartPanicWriterConfig, UsartRegisters};

/// The USART the panic handler writes to, which is the one the console uses.
const PANIC_USART: StaticRef<UsartRegisters> = USART1_BASE;

/// Panic handler.
#[panic_handler]
pub fn panic_fmt(info: &PanicInfo) -> ! {
    // To ensure that all dependencies are set up, we initialize all clocks
    // and GPIOs here in this function.
    //
    // Relying on `main.rs` to initialize clocks/gpios may result in the gpio
    // not being properly configured if the panic occurs early in `main.rs`.
    let rcc = stm32wle5jc::rcc::Rcc::new();
    let clocks: stm32wle5jc::clocks::Clocks<Stm32wle5jcSpecs> =
        stm32wle5jc::clocks::Clocks::new(&rcc);
    let syscfg = stm32wle5jc::syscfg::Syscfg::new();
    let exti = stm32wle5jc::exti::Exti::new(&syscfg);

    let gpio_ports = stm32wle5jc::gpio::GpioPorts::new(&clocks, &exti);
    gpio_ports.setup_circular_deps();
    gpio_ports
        .get_port_from_port_id(stm32wle5jc::gpio::PortId::B)
        .enable_clock();
    let pin = stm32wle5jc::gpio::Pin::new(PinId::PB05, &exti);
    pin.set_ports_ref(&gpio_ports);
    let led = &mut led::LedLow::new(&pin);

    // USART1: PB6=TX , PB7=RX
    gpio_ports.get_pin(PinId::PB06).map(|pin| {
        pin.set_mode(stm32wle5jc::gpio::Mode::AlternateFunctionMode);
        pin.set_alternate_function(stm32wle5jc::gpio::AlternateFunction::AF7);
    });

    gpio_ports.get_pin(PinId::PB07).map(|pin| {
        pin.set_mode(stm32wle5jc::gpio::Mode::AlternateFunctionMode);
        pin.set_alternate_function(stm32wle5jc::gpio::AlternateFunction::AF7);
    });

    // No `uart::Parameters` to pass in here: this chip's USART driver only
    // ever supports one configuration (115200 8N1, no hardware flow
    // control), so there is nothing for the panic handler to configure
    // differently.
    debug::panic::<_, Usart<'_>, _, _>(
        &mut [led],
        UsartPanicWriterConfig {
            registers: PANIC_USART,
            clock: PeripheralClock::new(PeripheralClockType::APB2(PCLK2::USART1), &clocks),
        },
        info,
        &cortexm4::support::nop,
        crate::PANIC_RESOURCES.get(),
    )
}
