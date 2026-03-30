// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

#![no_std]

pub mod chip;
pub mod tim;
pub mod usart;

use cortexm33::{initialize_ram_jump_to_main, unhandled_interrupt, CortexM33, CortexMVariant};

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

pub unsafe fn generic_init() {
    cortexm33::nvic::disable_all();
    cortexm33::nvic::clear_all_pending();
}
