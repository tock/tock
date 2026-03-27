// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Minimal peripheral crate for STM32U5xx.
//!
//! Only what is needed for first bring-up:
//! - chip glue
//! - chip-specific constants
//! - RCC + basic clocks (HSI / system / AHB / APB)
//! - GPIO
//! - NVIC + vector table
//! - USART
//! - TIM2 (basic delay)

#![no_std]

pub mod chip;
pub mod chip_specifics;
pub mod nvic;

pub mod clocks;
pub mod flash;
pub mod gpio;
pub mod rcc;
pub mod tim;
pub mod usart;
use cortexm33::{initialize_ram_jump_to_main, unhandled_interrupt, CortexM33, CortexMVariant};

extern "C" {
    // _estack is the initial stack pointer (defined in the linker script).
    // It is not a function; this is just to make the type system happy.
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
    unhandled_interrupt,           // MemManage (not present on all cores)
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
    // Basic NVIC sanitization at boot. `nvic` module should wrap the
    // cortexm NVIC helpers for STM32U5.
    cortexm33::nvic::disable_all();
    cortexm33::nvic::clear_all_pending();
    cortexm33::nvic::enable_all();
}
