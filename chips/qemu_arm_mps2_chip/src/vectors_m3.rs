// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! ARM Cortex-M vector table for the MPS2 AN385 (Cortex-M3) machine.
//!
//! There is no bootloader relocating the vector table on this QEMU-only
//! FPGA image: it is loaded and executed directly from address 0, which is
//! exactly where this table is placed by the linker script's `.vectors`
//! section. There are no documented silicon errata to apply, unlike real
//! hardware chip crates (this is a synthetic reference platform, not real
//! silicon).
//!
//! This is deliberately a concrete (non-generic) module, unlike
//! [`crate::chip::QemuArmMps2Chip`]: see the `cortex-m3`/`cortex-m4`
//! feature doc comment in this crate's `Cargo.toml` for why.

use cortexm3::{CortexM3, CortexMVariant, initialize_ram_jump_to_main, unhandled_interrupt};

extern "C" {
    // _estack is not really a function, but it makes the types work.
    // You should never actually invoke it!!
    fn _estack();
}

#[cfg_attr(
    all(target_arch = "arm", target_os = "none"),
    link_section = ".vectors"
)]
#[cfg_attr(all(target_arch = "arm", target_os = "none"), used)]
/// ARM Cortex-M Vector Table
pub static BASE_VECTORS: [unsafe extern "C" fn(); 16] = [
    _estack,                      // Stack Pointer
    initialize_ram_jump_to_main,  // Reset Handler
    unhandled_interrupt,          // NMI
    CortexM3::HARD_FAULT_HANDLER, // Hard Fault
    unhandled_interrupt,          // Memory Management Fault
    unhandled_interrupt,          // Bus Fault
    unhandled_interrupt,          // Usage Fault
    unhandled_interrupt,          // Reserved
    unhandled_interrupt,          // Reserved
    unhandled_interrupt,          // Reserved
    unhandled_interrupt,          // Reserved
    CortexM3::SVC_HANDLER,        // SVCall
    unhandled_interrupt,          // Reserved for Debug
    unhandled_interrupt,          // Reserved
    unhandled_interrupt,          // PendSv
    CortexM3::SYSTICK_HANDLER,    // SysTick
];

/// Number of NVIC external interrupt lines.
///
/// The an385 machine's NVIC is configured with
/// `qdev_prop_set_uint32(armv7m, "num-irq", 32)` in `hw/arm/mps2.c`; only a
/// handful are wired to real devices, but the vector table must cover the
/// full range.
const NUM_IRQS: usize = 32;

#[cfg_attr(all(target_arch = "arm", target_os = "none"), link_section = ".irqs")]
#[cfg_attr(all(target_arch = "arm", target_os = "none"), used)]
pub static IRQS: [unsafe extern "C" fn(); NUM_IRQS] = [CortexM3::GENERIC_ISR; NUM_IRQS];
