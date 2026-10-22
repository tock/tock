// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use cortexm33::{CortexM33, CortexMVariant, initialize_ram_jump_to_main, scb, unhandled_interrupt};

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
/// ARM Cortex M Vector Table
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

#[cfg_attr(all(target_arch = "arm", target_os = "none"), link_section = ".irqs")]
// used Ensures that the symbol is kept until the final binary
#[cfg_attr(all(target_arch = "arm", target_os = "none"), used)]
pub static IRQS: [unsafe extern "C" fn(); 69] = [CortexM33::GENERIC_ISR; 69];

unsafe fn read(addr: usize) -> u32 {
    unsafe { core::ptr::read_volatile(addr as *const u32) }
}

unsafe fn write(addr: usize, val: u32) {
    unsafe { core::ptr::write_volatile(addr as *mut u32, val) }
}

/// Apply the startup fixes from `SystemInit()` in Nordic's
/// `system_nrf5340_application.c` (nrfx), in the same order.
///
/// # Safety
///
/// These write to fixed, partly undocumented, register addresses. They are
/// only valid on the nRF5340 application core running in secure state.
pub(crate) unsafe fn fix_errata() {
    const CLOCK: usize = 0x5000_5000;
    const RESETREAS: usize = 0x5000_5400;
    const RESETREAS_RESETPIN: u32 = 1;

    // # Safety
    //
    // See function-level doc.
    unsafe {
        // The errata apply to the nRF5340 (FICR part code 7) and are gated on
        // its revision code, exactly as in nrfx.
        let nrf5340 = read(0x00FF_0130) == 7;
        let rev = read(0x00FF_0134);

        // Erratum 97.
        if nrf5340 && rev == 2 && read(0x5000_4A20) == 0 {
            write(0x5000_4A20, 0xD);
            write(0x5000_491C, 1);
            write(0x5000_491C, 0);
        }

        // Copy the analog trim values from FICR.TRIMCNF.
        for i in 0..32 {
            let addr = read(0x00FF_0300 + i * 8);
            if addr == 0xFFFF_FFFF {
                break;
            }
            write(addr as usize, read(0x00FF_0304 + i * 8));
        }

        // Erratum 64, which must come before erratum 42.
        if nrf5340 && rev == 2 {
            write(0x5000_470C, 0x29);
            write(0x5000_473C, 0x3);
        }

        // Erratum 42: set HFCLKCTRL to divide by 2 (64 MHz).
        if nrf5340 && rev == 2 {
            write(0x5003_9530, 0xBEEF_0044);
            write(CLOCK + 0x558, 1);
        }

        // Erratum 46.
        if nrf5340 && (rev == 2 || rev == 3) {
            write(0x5003_254C, 0);
        }

        let pin_reset = read(RESETREAS) & RESETREAS_RESETPIN != 0;

        // Erratum 49: clear POWER EVENTS_SLEEPENTER and EVENTS_SLEEPEXIT.
        if nrf5340 && rev == 2 && pin_reset {
            write(CLOCK + 0x114, 0);
            write(CLOCK + 0x118, 0);
        }

        // Erratum 55.
        if nrf5340 && pin_reset {
            write(RESETREAS, !RESETREAS_RESETPIN);
        }

        // Erratum 69.
        if nrf5340 && rev == 2 {
            write(0x5000_470C, 0x65);
        }

        // Erratum 140: run the LFCLK from LFSYNT once, then back to LFRC.
        if nrf5340 && read(0x5003_2420) & 0x8000_0000 != 0 {
            write(CLOCK + 0x518, 3);
            write(CLOCK + 0x008, 1);
            while read(CLOCK + 0x104) == 0 {}
            write(CLOCK + 0x104, 0);
            write(CLOCK + 0x00C, 1);
            write(CLOCK + 0x518, 1);
        }

        // Erratum 160.
        if nrf5340 && !matches!(rev, 2..=4) {
            write(0x5000_470C, 0x7E);
            write(0x5000_493C, 0x7E);
            write(0x5000_2118, 0x7F);
            write(0x5003_9E04, 0);
            write(0x5003_9E08, 0);
            write(0x5010_1110, 0);
            write(0x5000_2124, 0);
            write(0x5000_212C, 0);
            write(0x5020_12A0, 0);
        }

        // Load the APPROTECT and SECUREAPPROTECT settings from UICR into
        // CTRLAP. Debug access stays open only if UICR holds the Unprotected
        // value (0x50FA50FA).
        write(0x5000_6544, read(0x00FF_8000));
        write(0x5000_654C, read(0x00FF_801C));
    }
}

/// Explicitly tell the core where Tock's vector table is located.
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
