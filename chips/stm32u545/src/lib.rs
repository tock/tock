// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

#![no_std]

pub use stm32u5xx::{chip, exti, gpio, rcc, tim, usart, dma, Stm32u5xxPeripherals};

use cortexm33::{CortexMVariant, CortexM33};

pub unsafe fn init() {
    stm32u5xx::init();
}

#[cfg_attr(all(target_arch = "arm", target_os = "none"), used)]
#[cfg_attr(all(target_arch = "arm", target_os = "none"), link_section = ".irqs")]
pub static IRQS: [unsafe extern "C" fn(); 125] = [<CortexM33 as CortexMVariant>::GENERIC_ISR; 125];
