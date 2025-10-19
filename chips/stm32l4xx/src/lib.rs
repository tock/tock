// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Author: Kamil Duljas <kamil.duljas@gmail.com>

//! Peripheral implementations for the STM32L4xx MCU.
//!
//! STM32L476RG: <https://www.st.com/en/microcontrollers/stm32l4.html>

#![no_std]

pub mod chip;
pub mod chip_specific;
pub mod nvic;

// Clocks
pub mod clocks;

// Peripherials
pub mod exti;
pub mod flash;
pub mod gpio;
pub mod pwr;
pub mod rcc;
pub mod syscfg;
pub mod usart;

use cortexm4f::{initialize_ram_jump_to_main, unhandled_interrupt, CortexM4F, CortexMVariant};

extern "C" {
    // _estack is not really a function, but it makes the types work
    // You should never actually invoke it!!
    fn _estack();
}

#[cfg_attr(
    all(target_arch = "arm", target_os = "none"),
    link_section = ".vectors"
)]
// used Ensures that the symbol is kept until the final binary
#[cfg_attr(all(target_arch = "arm", target_os = "none"), used)]
pub static BASE_VECTORS: [unsafe extern "C" fn(); 16] = [
    _estack,
    initialize_ram_jump_to_main,
    unhandled_interrupt,           // NMI
    CortexM4F::HARD_FAULT_HANDLER, // Hard Fault
    unhandled_interrupt,           // MemManage
    unhandled_interrupt,           // BusFault
    unhandled_interrupt,           // UsageFault
    unhandled_interrupt,
    unhandled_interrupt,
    unhandled_interrupt,
    unhandled_interrupt,
    CortexM4F::SVC_HANDLER, // SVC
    unhandled_interrupt,    // DebugMon
    unhandled_interrupt,
    unhandled_interrupt,        // PendSV
    CortexM4F::SYSTICK_HANDLER, // SysTick
];

pub unsafe fn init() {
    cortexm4f::nvic::disable_all();
    cortexm4f::nvic::clear_all_pending();
    cortexm4f::nvic::enable_all();
}
