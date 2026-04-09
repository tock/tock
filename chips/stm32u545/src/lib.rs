// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2026.

#![no_std]

pub use stm32u5xx::{chip, dma, exti, gpio, rcc, tim, usart, init_dma1, init_exti, init_usart1, Stm32u5xxPeripherals};

use cortexm33::{CortexM33, CortexMVariant};

pub unsafe fn init() {
    stm32u5xx::init();
}

#[cfg_attr(all(target_arch = "arm", target_os = "none"), used)]
#[cfg_attr(all(target_arch = "arm", target_os = "none"), link_section = ".irqs")]
pub static IRQS: [unsafe extern "C" fn(); 125] = [<CortexM33 as CortexMVariant>::GENERIC_ISR; 125];
