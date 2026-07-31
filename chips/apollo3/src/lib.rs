// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Peripheral implementations for the Apollo3 MCU.

#![no_std]
// Fixes this error introduced in nightly-2026-07:
//
// ```
// error: queries overflow the depth limit!
//   |
//   = help: consider increasing the recursion limit by adding a `#![recursion_limit = "256"]` attribute to your crate (`apollo3`)
//   = note: query depth increased by 130 when simplifying constant for the type system `mcuctrl::_`
// ```
#![recursion_limit = "256"]

// Peripherals
pub mod ble;
pub mod cachectrl;
pub mod chip;
pub mod clkgen;
pub mod flashctrl;
pub mod gpio;
pub mod iom;
pub mod ios;
pub mod mcuctrl;
pub mod nvic;
pub mod pwrctrl;
pub mod stimer;
pub mod uart;

use cortexm4f::{CortexM4F, CortexMVariant, initialize_ram_jump_to_main, scb, unhandled_interrupt};

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

#[cfg_attr(
    all(target_arch = "arm", target_os = "none"),
    link_section = ".vectors"
)]
// used Ensures that the symbol is kept until the final binary
#[cfg_attr(all(target_arch = "arm", target_os = "none"), used)]
pub static IRQS: [unsafe extern "C" fn(); 32] = [CortexM4F::GENERIC_ISR; 32];

// The Patch table.
//
// The patch table should pad the vector table size to a total of 64 entries
// (16 core + 48 periph) such that code begins at offset 0x100.
#[cfg_attr(
    all(target_arch = "arm", target_os = "none"),
    link_section = ".vectors"
)]
// used Ensures that the symbol is kept until the final binary
#[cfg_attr(all(target_arch = "arm", target_os = "none"), used)]
pub static PATCH: [unsafe extern "C" fn(); 16] = [unhandled_interrupt; 16];

// The SVC call in this function means that we need to ensure it's inlined in
// `main()` otherwise we end up with a clobbered stack.
#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
#[inline(always)]
pub unsafe fn actually_disable_fpu() {
    use core::arch::asm;

    // This ensures the FPU is actually disabled
    asm!("svc 0xff", out("r0") _, out("r1") _, out("r2") _, out("r3") _, out("r12") _);
}

// Mock implementation for tests
#[cfg(not(any(doc, all(target_arch = "arm", target_os = "none"))))]
pub unsafe fn actually_disable_fpu() {
    // Prevent unused code warning.
    scb::disable_fpca();

    unimplemented!()
}
