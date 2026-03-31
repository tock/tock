// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

#![no_std]

use cortexm33::{unhandled_interrupt, CortexM33, CortexMVariant};

pub use stm32u5xx::{chip, generic_init, gpio, rcc, tim, usart, Stm32u5xxPeripherals};

// STM32U545 has a total of 126 interrupts
#[cfg_attr(all(target_arch = "arm", target_os = "none"), link_section = ".irqs")]
#[cfg_attr(all(target_arch = "arm", target_os = "none"), used)]
pub static IRQS: [unsafe extern "C" fn(); 126] = {
    let mut table = [unhandled_interrupt as unsafe extern "C" fn(); 126];

    // Index 45 is TIM2
    table[45] = CortexM33::GENERIC_ISR;
    // Index 61 is USART1
    table[61] = CortexM33::GENERIC_ISR;

    table
};

pub unsafe fn init() {
    generic_init();
}
