// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2026.

#![no_std]

pub mod adc;
pub mod chip;
pub mod dac;
pub mod dma;
pub mod exti;
pub mod gpio;
pub mod nvic;
pub mod pwr;
pub mod rcc;
pub mod rtc;
pub mod tim;
pub mod usart;

use cortexm33::{CortexM33, CortexMVariant, initialize_ram_jump_to_main, unhandled_interrupt};

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
    _estack,                       // 0x00: Initial stack pointer
    initialize_ram_jump_to_main,   // 0x04: Reset
    unhandled_interrupt,           // 0x08: NMI
    CortexM33::HARD_FAULT_HANDLER, // 0x0C: HardFault
    unhandled_interrupt,           // 0x10: MemManage
    unhandled_interrupt,           // 0x14: BusFault
    unhandled_interrupt,           // 0x18: UsageFault
    unhandled_interrupt,           // 0x1C: SecureFault
    unhandled_interrupt,           // 0x20: Reserved
    unhandled_interrupt,           // 0x24: Reserved
    unhandled_interrupt,           // 0x28: Reserved
    CortexM33::SVC_HANDLER,        // 0x2C: SVCall
    unhandled_interrupt,           // 0x30: Debug monitor
    unhandled_interrupt,           // 0x34: Reserved
    unhandled_interrupt,           // 0x38: PendSV
    CortexM33::SYSTICK_HANDLER,    // 0x3C: SysTick
];
