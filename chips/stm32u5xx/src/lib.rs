// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2026.

#![no_std]

pub mod chip;
pub mod dma;
pub mod exti;
pub mod gpio;
pub mod rcc;
pub mod tim;
pub mod usart;

use cortexm33::{initialize_ram_jump_to_main, unhandled_interrupt, CortexM33, CortexMVariant};
use kernel::utilities::StaticRef;

extern "C" {
    // _estack is the initial stack pointer (defined in the linker script).
    fn _estack();
}

#[cfg_attr(
    all(target_arch = "arm", target_os = "none"),
    link_section = ".vectors"
)]
#[cfg_attr(all(target_arch = "arm", target_os = "none"), used)]
pub static BASE_VECTORS: [unsafe extern "C" fn(); 16] = [
    _estack,                       // Initial stack pointer
    initialize_ram_jump_to_main,   // Reset
    unhandled_interrupt,           // NMI
    CortexM33::HARD_FAULT_HANDLER, // HardFault
    unhandled_interrupt,           // MemManage
    unhandled_interrupt,           // BusFault
    unhandled_interrupt,           // UsageFault
    unhandled_interrupt,           // Reserved
    unhandled_interrupt,           // Reserved
    unhandled_interrupt,           // Reserved
    unhandled_interrupt,           // Reserved
    CortexM33::SVC_HANDLER,        // SVCall
    unhandled_interrupt,           // Debug monitor
    unhandled_interrupt,           // Reserved
    unhandled_interrupt,           // PendSV
    CortexM33::SYSTICK_HANDLER,    // SysTick
];

pub unsafe fn init() {
    cortexm33::nvic::disable_all();
    cortexm33::nvic::clear_all_pending();
    cortexm33::nvic::enable_all();
}

pub struct Stm32u5xxPeripherals<'a> {
    pub rcc: rcc::Rcc,
    pub exti: &'a exti::Exti<'a>,
    pub dma1: &'a dma::Dma<'a>,
    pub gpio_a: gpio::Port<'a>,
    pub gpio_c: gpio::Port<'a>,
    pub usart1: &'a usart::Usart<'a>,
    pub tim2: tim::Tim2<'a>,
}

impl<'a> Stm32u5xxPeripherals<'a> {
    pub unsafe fn new(
        exti: &'a exti::Exti<'a>,
        dma1: &'a dma::Dma<'a>,
        usart1: &'a usart::Usart<'a>,
    ) -> Self {
        Self {
            rcc: rcc::Rcc::new(StaticRef::new(0x46020C00 as *const rcc::RccRegisters)),
            exti,
            dma1,
            gpio_a: gpio::Port::new(
                StaticRef::new(0x52020000 as *const gpio::GpioRegisters),
                exti,
                0,
            ),
            gpio_c: gpio::Port::new(
                StaticRef::new(0x52020800 as *const gpio::GpioRegisters),
                exti,
                2,
            ),
            usart1,
            tim2: tim::Tim2::new(StaticRef::new(0x50000000 as *const tim::TimRegisters)),
        }
    }
}
