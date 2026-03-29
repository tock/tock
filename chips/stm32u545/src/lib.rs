// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

#![no_std]

use cortexm33::unhandled_interrupt;

pub use stm32u5xx::{chip, generic_init, usart};

// STM32U545 has a total of 126 interrupts as per our research
#[cfg_attr(all(target_arch = "arm", target_os = "none"), link_section = ".irqs")]
#[cfg_attr(all(target_arch = "arm", target_os = "none"), used)]
pub static IRQS: [unsafe extern "C" fn(); 126] = [unhandled_interrupt; 126];

pub unsafe fn init() {
    generic_init();
}
