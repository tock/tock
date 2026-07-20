// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Infineon Technologies AG 2026.

//! PSC3 support crate

#![no_std]
// Increase the recursion limit for SRSS Registers
#![recursion_limit = "512"]

use cortexm33::{
    CortexM33NonSecure, CortexM33Secure, CortexMVariant, initialize_ram_jump_to_main,
    unhandled_interrupt,
};

extern "C" {
    // _estack is not really a function, but it makes the types work
    // You should never actually invoke it!!
    fn _estack();
}

#[cfg_attr(
    all(target_arch = "arm", target_os = "none"),
    link_section = ".vectors"
)]
pub static BASE_VECTORS_SECURE: [unsafe extern "C" fn(); 16] = [
    _estack,
    initialize_ram_jump_to_main,
    unhandled_interrupt,                 // NMI
    CortexM33Secure::HARD_FAULT_HANDLER, // Hard Fault
    unhandled_interrupt,                 // MemManage
    unhandled_interrupt,                 // BusFault
    unhandled_interrupt,                 // UsageFault
    unhandled_interrupt,
    unhandled_interrupt,
    unhandled_interrupt,
    unhandled_interrupt,
    CortexM33Secure::SVC_HANDLER, // SVC
    unhandled_interrupt,          // DebugMon
    unhandled_interrupt,
    unhandled_interrupt,              // PendSV
    CortexM33Secure::SYSTICK_HANDLER, // SysTick
];

pub static BASE_VECTORS_NON_SECURE: [unsafe extern "C" fn(); 16] = [
    _estack,
    initialize_ram_jump_to_main,
    unhandled_interrupt,                    // NMI
    CortexM33NonSecure::HARD_FAULT_HANDLER, // Hard Fault
    unhandled_interrupt,                    // MemManage
    unhandled_interrupt,                    // BusFault
    unhandled_interrupt,                    // UsageFault
    unhandled_interrupt,
    unhandled_interrupt,
    unhandled_interrupt,
    unhandled_interrupt,
    CortexM33NonSecure::SVC_HANDLER, // SVC
    unhandled_interrupt,             // DebugMon
    unhandled_interrupt,
    unhandled_interrupt,                 // PendSV
    CortexM33NonSecure::SYSTICK_HANDLER, // SysTick
];

#[cfg_attr(all(target_arch = "arm", target_os = "none"), link_section = ".irqs")]
pub static IRQS_SECURE: [unsafe extern "C" fn(); 140] = [CortexM33Secure::GENERIC_ISR; 140];

#[cfg_attr(all(target_arch = "arm", target_os = "none"), link_section = ".irqs")]
pub static IRQS_NON_SECURE: [unsafe extern "C" fn(); 140] = [CortexM33Secure::GENERIC_ISR; 140];

pub mod chip;
pub mod chip_init;
pub mod cpuss_ppu;
pub mod flashc;
pub mod gpio;
mod gpio_registers;
mod hsiom_registers;
pub mod icache;
pub mod interrupts;
pub mod peri;
pub mod peri_clk;
pub mod pwrmode;
pub mod ramc_ppu;
pub mod scb;
mod scb_registers;
pub mod srss;
mod srss_registers;
pub mod tcpwm;
