// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use cortexm4f::{CortexM4F, CortexMVariant, initialize_ram_jump_to_main, scb, unhandled_interrupt};

/*
 * Adapted from crt1.c which was relicensed by the original author from
 * GPLv3 to Apache 2.0.
 * The original version of the file, under GPL can be found at
 * https://github.com/SoftwareDefinedBuildings/stormport/blob/rebase0/tos/platforms/storm/stormcrt1.c
 *
 * Copyright 2016, Michael Andersen <m.andersen@eecs.berkeley.edu>
 */

// Get the `_estack` symbol from the linker.
//
// This variable must never be read.
//
// SAFETY: This is a valid and unique linker symbol. `[u8; 0]` is as close as
// the type as we can get (this is not a valid memory location to access as any
// type).
unsafe extern "C" {
    static _estack: [u8; 0];
}

// Ensure the address where the stack starts (i.e., the top of the stack or the
// end of the stack memory range) is inserted at the start of the `.vectors`
// section.
//
// Inserting the address of this symbol in this section is hard to do in any
// other valid way. To use the
// [`link_section`](https://doc.rust-lang.org/reference/abi.html#the-link_section-attribute)
// attribute, the variable must be static. However, a static variable cannot be
// of type `*const u8` because it is not Sync. A static can be of type `unsafe
// extern "C" fn()`, however, it is not correct to define `_stack` as a `unsafe
// extern "C" fn()`, because it does point to a function. Making the static
// object a `usize` is possible, however, converting the linker symbol address
// to a usize in a const environment is not. So, we are left using
// `global_asm!()` to define the constant in the linker section.
core::arch::global_asm!(
    "
.section .vectors
.word {estack}
    ",
    estack = sym _estack
);

/// ARM Cortex-M Vector Table
///
/// # Safety
///
/// - `link_section = ".vectors"`: We must put this array of function pointers
///   in the vector table to be compatible with the Cortex-M hardware when the
///   MCU powers on. This array of function pointers is read-only and this
///   section is placed in the .text segment.
// The `used` attribute ensures that the symbol is kept until the final binary.
#[cfg_attr(
    all(target_arch = "arm", target_os = "none"),
    unsafe(link_section = ".vectors")
)]
#[cfg_attr(all(target_arch = "arm", target_os = "none"), used)]
pub static BASE_VECTORS: [unsafe extern "C" fn(); 15] = [
    // Reset Handler
    initialize_ram_jump_to_main,
    // NMI
    unhandled_interrupt,
    // Hard Fault
    CortexM4F::HARD_FAULT_HANDLER,
    // Memory Management Fault
    unhandled_interrupt,
    // Bus Fault
    unhandled_interrupt,
    // Usage Fault
    unhandled_interrupt,
    // Reserved
    unhandled_interrupt,
    // Reserved
    unhandled_interrupt,
    // Reserved
    unhandled_interrupt,
    // Reserved
    unhandled_interrupt,
    // SVCall
    CortexM4F::SVC_HANDLER,
    // Reserved for Debug
    unhandled_interrupt,
    // Reserved
    unhandled_interrupt,
    // PendSv
    unhandled_interrupt,
    // SysTick
    CortexM4F::SYSTICK_HANDLER,
];

/// nRF52 IRQ Function Pointers
///
/// # Safety
///
/// - `link_section = ".irq"`: We must put this array of function pointers at
///   the beginning of the .text segment after the Cortex-M vector table. This
///   array of function pointers is read-only and this section is placed in the
///   .text segment.
// The `used` attribute ensures that the symbol is kept until the final binary.
#[cfg_attr(
    all(target_arch = "arm", target_os = "none"),
    unsafe(link_section = ".irqs")
)]
#[cfg_attr(all(target_arch = "arm", target_os = "none"), used)]
pub static IRQS: [unsafe extern "C" fn(); 80] = [CortexM4F::GENERIC_ISR; 80];

/// Apply fixes for various nRF52 errata
///
/// # Safety
///
/// Fixing these errata requires writing to various memory locations. These
/// operations are safe as long as this is only run on an nRF52 MCU.
pub(crate) unsafe fn fix_errata() {
    // SAFETY: Same as the function.
    unsafe {
        // Apply early initialization workarounds for anomalies documented on
        // 2015-12-11 nRF52832 Errata v1.2
        // http://infocenter.nordicsemi.com/pdf/nRF52832_Errata_v1.2.pdf

        // Workaround for Errata 12
        // "COMP: Reference ladder not correctly callibrated" found at the Errate doc
        core::ptr::write_volatile(
            0x40013540i32 as *mut u32,
            (core::ptr::read_volatile(0x10000324i32 as *mut u32) & 0x1f00u32) >> 8i32,
        );

        // Workaround for Errata 16
        // "System: RAM may be corrupt on wakeup from CPU IDLE" found at the Errata doc
        core::ptr::write_volatile(0x4007c074i32 as *mut u32, 3131961357u32);

        // Workaround for Errata 31
        // "CLOCK: Calibration values are not correctly loaded from FICR at reset"
        // found at the Errata doc
        core::ptr::write_volatile(
            0x4000053ci32 as *mut u32,
            (core::ptr::read_volatile(0x10000244i32 as *mut u32) & 0xe000u32) >> 13i32,
        );

        // Only needed for preview hardware
        // // Workaround for Errata 32
        // // "DIF: Debug session automatically enables TracePort pins" found at the Errata doc
        // //    CoreDebug->DEMCR &= ~CoreDebug_DEMCR_TRCENA_Msk;
        // *(0xe000edfcu32 as (*mut u32)) &= !0x01000000,

        // Workaround for Errata 36
        // "CLOCK: Some registers are not reset when expected" found at the Errata doc
        //    NRF_CLOCK->EVENTS_DONE = 0;
        //    NRF_CLOCK->EVENTS_CTTO = 0;
        //    NRF_CLOCK->CTIV = 0;
        // }

        // Workaround for Errata 37
        // "RADIO: Encryption engine is slow by default" found at the Errata document doc
        core::ptr::write_volatile(0x400005a0i32 as *mut u32, 0x3u32);

        // Workaround for Errata 57
        // "NFCT: NFC Modulation amplitude" found at the Errata doc
        core::ptr::write_volatile(0x40005610i32 as *mut u32, 0x5u32);
        core::ptr::write_volatile(0x40005688i32 as *mut u32, 0x1u32);
        core::ptr::write_volatile(0x40005618i32 as *mut u32, 0x0u32);
        core::ptr::write_volatile(0x40005614i32 as *mut u32, 0x3fu32);

        // Workaround for Errata 66
        // "TEMP: Linearity specification not met with default settings" found at the Errata doc
        //     NRF_TEMP->A0 = NRF_FICR->TEMP.A0;
        //     NRF_TEMP->A1 = NRF_FICR->TEMP.A1;
        //     NRF_TEMP->A2 = NRF_FICR->TEMP.A2;
        //     NRF_TEMP->A3 = NRF_FICR->TEMP.A3;
        //     NRF_TEMP->A4 = NRF_FICR->TEMP.A4;
        //     NRF_TEMP->A5 = NRF_FICR->TEMP.A5;
        //     NRF_TEMP->B0 = NRF_FICR->TEMP.B0;
        //     NRF_TEMP->B1 = NRF_FICR->TEMP.B1;
        //     NRF_TEMP->B2 = NRF_FICR->TEMP.B2;
        //     NRF_TEMP->B3 = NRF_FICR->TEMP.B3;
        //     NRF_TEMP->B4 = NRF_FICR->TEMP.B4;
        //     NRF_TEMP->B5 = NRF_FICR->TEMP.B5;
        //     NRF_TEMP->T0 = NRF_FICR->TEMP.T0;
        //     NRF_TEMP->T1 = NRF_FICR->TEMP.T1;
        //     NRF_TEMP->T2 = NRF_FICR->TEMP.T2;
        //     NRF_TEMP->T3 = NRF_FICR->TEMP.T3;
        //     NRF_TEMP->T4 = NRF_FICR->TEMP.T4;
        // }

        // Workaround for Errata 108
        // "RAM: RAM content cannot be trusted upon waking up from System ON Idle
        // or System OFF mode" found at the Errata doc
        core::ptr::write_volatile(
            0x40000ee4i32 as *mut u32,
            core::ptr::read_volatile(0x10000258i32 as *mut u32) & 0x4fu32,
        );
    }
}

/// Explicitly tell the core where Tock's vector table is located.
///
/// If Tock is the
/// only thing on the chip then this is effectively a no-op. If, however, there is
/// a bootloader present then we want to ensure that the vector table is set
/// correctly for Tock. The bootloader _may_ set this for us, but it may not
/// so that any errors early in the Tock boot process trap back to the bootloader.
/// To be safe we unconditionally set the vector table.
pub(crate) fn initialize_vector_table() {
    // # Safety
    //
    // The vector table must setup function pointers for the thumb core
    // correctly. Because `BASE_VECTORS` is the correct data type this is
    // safe.
    unsafe {
        scb::set_vector_table_offset(BASE_VECTORS.as_ptr().cast::<()>());
    }
}
