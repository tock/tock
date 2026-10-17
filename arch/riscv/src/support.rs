// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Core low-level operations.

use crate::csr::{CSR, mstatus::mstatus};
use kernel::capabilities;

#[cfg(riscv)]
#[inline(always)]
/// NOP instruction
pub fn nop() {
    use core::arch::asm;
    unsafe {
        asm!("nop", options(nomem, nostack, preserves_flags));
    }
}

#[cfg(riscv)]
#[inline(always)]
/// Wait For Interrupt (WFI) instruction.
pub unsafe fn wfi() {
    use core::arch::asm;
    asm!("wfi", options(nomem, nostack));
}

/// Single-core critical section operation
pub fn with_interrupts_disabled<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    // Read the mstatus MIE field and disable machine mode interrupts
    // atomically
    //
    // The result will be the original value of [`mstatus::mie`],
    // shifted to the proper position in [`mstatus`].
    let original_mie: usize = CSR
        .mstatus
        .read_and_clear_bits(mstatus::mie.mask << mstatus::mie.shift)
        & mstatus::mie.mask << mstatus::mie.shift;

    // Machine mode interrupts are disabled, execute the (uninterruptible)
    // function
    let res = f();

    // If [`mstatus::mie`] was set before, set it again. Otherwise,
    // this function will be a nop.
    CSR.mstatus.read_and_set_bits(original_mie);

    res
}

/// Deliberately trigger a hard fault (undefined instruction) on the chip.
#[cfg(riscv)]
pub fn hardfault(_cap: &dyn capabilities::CpuControlCapability) -> ! {
    // SAFETY: This does not use any registers or memory, it just crashes.
    unsafe {
        core::arch::asm!("unimp", options(noreturn, nomem, nostack));
    }
}

// Mock implementations for tests on Travis-CI.
#[cfg(not(riscv))]
pub fn nop() {
    unimplemented!()
}

#[cfg(not(riscv))]
pub unsafe fn wfi() {
    unimplemented!()
}

#[cfg(not(riscv))]
pub fn hardfault(_cap: &dyn capabilities::CpuControlCapability) -> ! {
    unimplemented!()
}
