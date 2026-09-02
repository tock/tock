// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Chip support specific to the ARM MPS2 AN386 FPGA image.
//!
//! Nothing relocates the vector table: the image executes directly from
//! address 0, where the linker script places `.vectors`.

#![no_std]

/// This image's CPU core.
///
/// Boards name their core through here rather than reaching for `cortexm4`
/// directly, because nothing else in this crate is referenced by name and a
/// dependency nothing names is dropped before the linker sees the vector
/// table below.
pub use cortexm4::CortexM4;

use cortexm4::{CortexMVariant, initialize_ram_jump_to_main, unhandled_interrupt};

extern "C" {
    // _estack is not really a function, but it makes the types work.
    // You should never actually invoke it!!
    //
    // Keep this in the chip crate: `process_console` declares `_estack` as
    // data, and LTO can't hold both types in one unit -- it renames ours to
    // `_estack.1` and the link then fails, blaming an unrelated line.
    fn _estack();
}

#[cfg_attr(
    all(target_arch = "arm", target_os = "none"),
    link_section = ".vectors"
)]
#[cfg_attr(all(target_arch = "arm", target_os = "none"), used)]
// Stable, per-image name so each board's `layout.ld` can assert that its
// own vector table -- not the other image's -- was linked in.
#[unsafe(export_name = "mps2_an386_vector_table")]
/// ARM Cortex-M Vector Table
pub static BASE_VECTORS: [unsafe extern "C" fn(); 16] = [
    _estack,                      // Stack Pointer
    initialize_ram_jump_to_main,  // Reset Handler
    unhandled_interrupt,          // NMI
    CortexM4::HARD_FAULT_HANDLER, // Hard Fault
    unhandled_interrupt,          // Memory Management Fault
    unhandled_interrupt,          // Bus Fault
    unhandled_interrupt,          // Usage Fault
    unhandled_interrupt,          // Reserved
    unhandled_interrupt,          // Reserved
    unhandled_interrupt,          // Reserved
    unhandled_interrupt,          // Reserved
    CortexM4::SVC_HANDLER,        // SVCall
    unhandled_interrupt,          // Reserved for Debug
    unhandled_interrupt,          // Reserved
    unhandled_interrupt,          // PendSv
    CortexM4::SYSTICK_HANDLER,    // SysTick
];

/// Number of NVIC external interrupt lines.
///
/// The an386 machine's NVIC is configured with
/// `qdev_prop_set_uint32(armv7m, "num-irq", 32)` in `hw/arm/mps2.c`; only a
/// handful are wired to real devices, but the vector table must cover the
/// full range.
const NUM_IRQS: usize = 32;

#[cfg_attr(all(target_arch = "arm", target_os = "none"), link_section = ".irqs")]
#[cfg_attr(all(target_arch = "arm", target_os = "none"), used)]
pub static IRQS: [unsafe extern "C" fn(); NUM_IRQS] = [CortexM4::GENERIC_ISR; NUM_IRQS];
