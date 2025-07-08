// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Peripheral implementations for the Apollo3 MCU.

#![no_std]

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

use cortexm4f::{initialize_ram_jump_to_main, scb, unhandled_interrupt, CortexM4F, CortexMVariant};

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
pub unsafe fn init() {
    use core::arch::asm;
    let cache_ctrl = crate::cachectrl::CacheCtrl::new();
    cache_ctrl.enable_cache();

    // Explicitly tell the core where Tock's vector table is located. If Tock is the
    // only thing on the chip then this is effectively a no-op. If, however, there is
    // a bootloader present then we want to ensure that the vector table is set
    // correctly for Tock. The bootloader _may_ set this for us, but it may not
    // so that any errors early in the Tock boot process trap back to the bootloader.
    // To be safe we unconditionally set the vector table.
    scb::set_vector_table_offset(BASE_VECTORS.as_ptr() as *const ());

    // Disable the FPU (it might be enalbed by a prior stage)
    scb::disable_fpca();

    // This ensures the FPU is actually disabled
    asm!("svc 0xff", out("r0") _, out("r1") _, out("r2") _, out("r3") _, out("r12") _);

    cortexm4f::nvic::disable_all();
    cortexm4f::nvic::clear_all_pending();
    cortexm4f::nvic::enable_all();
}

// Mock implementation for tests
#[cfg(not(any(doc, all(target_arch = "arm", target_os = "none"))))]
pub unsafe fn init() {
    // Prevent unused code warning.
    scb::disable_fpca();

    unimplemented!()
}
