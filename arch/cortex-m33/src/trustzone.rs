// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Armv8-M Security Extension (TrustZone) support.
//!
//! [`is_secure`] tells code which security state it runs in. The other
//! functions are for a secure stage that hands the core to a non-secure image.

use kernel::utilities::StaticRef;
use kernel::utilities::registers::ReadWrite;
use kernel::utilities::registers::interfaces::{Readable, Writeable};

// SAU_CTRL bit 1, ALLNS: with the SAU disabled, memory the IDAU does not mark
// secure is non-secure.
const SAU_CTRL: StaticRef<ReadWrite<u32>> =
    unsafe { StaticRef::new(0xE000EDD0 as *const ReadWrite<u32>) };
const AIRCR: StaticRef<ReadWrite<u32>> =
    unsafe { StaticRef::new(0xE000ED0C as *const ReadWrite<u32>) };
const NVIC_ITNS: StaticRef<[ReadWrite<u32>; 16]> =
    unsafe { StaticRef::new(0xE000E380 as *const [ReadWrite<u32>; 16]) };

/// Whether the core runs in secure state.
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub fn is_secure() -> bool {
    let address = is_secure as fn() -> bool as usize;
    let result: usize;
    // # Safety
    //
    // TT only reads the security attributes of an address. Its S bit (22) is
    // set only when executed in secure state on secure memory, so testing
    // this code's own address gives the current state.
    unsafe {
        core::arch::asm!(
            "tt {result}, {address}",
            result = out(reg) result,
            address = in(reg) address,
            options(nomem, nostack, preserves_flags),
        );
    }
    result & (1 << 22) != 0
}

#[cfg(not(all(target_arch = "arm", target_os = "none")))]
pub fn is_secure() -> bool {
    unimplemented!()
}

/// Leave memory security to the IDAU, by disabling the SAU with all regions
/// non-secure.
pub fn sau_all_nonsecure() {
    SAU_CTRL.set(1 << 1);
}

/// Deliver interrupt `irq` to non-secure state.
pub fn interrupt_target_nonsecure(irq: usize) {
    let itns = &NVIC_ITNS[irq / 32];
    itns.set(itns.get() | (1 << (irq % 32)));
}

/// Deliver BusFault, HardFault and NMI to non-secure state (AIRCR.BFHFNMINS).
pub fn faults_target_nonsecure() {
    let prigroup = AIRCR.get() & 0x0700;
    AIRCR.set(0x05FA_0000 | prigroup | (1 << 13));
}

/// Start the non-secure image whose vector table is at `vector_table`: point
/// the non-secure VTOR at it, load its initial stack pointer into MSP_NS and
/// branch to its reset handler in non-secure state.
///
/// # Safety
///
/// `vector_table` must hold a valid non-secure image, in memory and with
/// peripherals that the SPU or IDAU make non-secure. This never returns, and
/// leaves secure code unreachable.
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub unsafe fn jump_to_nonsecure(vector_table: usize) -> ! {
    // The non-secure SCB's VTOR, as seen from secure state.
    const VTOR_NS: StaticRef<ReadWrite<u32>> =
        unsafe { StaticRef::new(0xE002ED08 as *const ReadWrite<u32>) };
    VTOR_NS.set(vector_table as u32);
    // # Safety
    //
    // The caller guarantees the vector table is valid.
    let (stack, reset) = unsafe {
        (
            core::ptr::read_volatile(vector_table as *const u32),
            core::ptr::read_volatile((vector_table + 4) as *const u32),
        )
    };
    // # Safety
    //
    // The encoded instructions are `msr msp_ns, r0` and `bxns r1`, which the
    // assembler accepts only with the security extension enabled. BXNS needs
    // bit 0 of the target clear to enter non-secure state.
    unsafe {
        core::arch::asm!(
            ".inst.w 0xf3808888",
            ".inst.n 0x470c",
            in("r0") stack,
            in("r1") reset & !1,
            options(noreturn),
        );
    }
}

#[cfg(not(all(target_arch = "arm", target_os = "none")))]
pub unsafe fn jump_to_nonsecure(_vector_table: usize) -> ! {
    unimplemented!()
}
