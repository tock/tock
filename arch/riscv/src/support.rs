// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Core low-level operations.

use crate::csr::{mstatus::mstatus, CSR};
use core::ops::FnOnce;
use core::ptr::NonNull;

#[cfg(any(doc, target_os = "none"))]
#[inline(always)]
/// NOP instruction
pub fn nop() {
    use core::arch::asm;
    unsafe {
        asm!("nop", options(nomem, nostack, preserves_flags));
    }
}

#[cfg(any(doc, target_os = "none"))]
#[inline(always)]
/// WFI instruction
pub unsafe fn wfi() {
    use core::arch::asm;
    asm!("wfi", options(nomem, nostack));
}

#[inline(always)]
/// sfence.vma instruction
pub fn sfence_vma() {
    use core::arch::asm;
    unsafe {
        asm!("sfence.vma", options(nomem, nostack));
    }
}

/// sfence.vma instruction with arguments to invalidate a single ASID
pub fn sfence_vma_asid(_asid: usize) {
    // First argument is address, second is ASID. An argument with _register_ zero applies to all
    // addresses / ASIDs. Another register with a _value_ of 0 will select the first page or
    // ASID 0.
    #[cfg(target_os = "none")]
    unsafe {
        use core::arch::asm;
        asm!("sfence.vma x0, {asid}", asid = in(reg) _asid, options(nomem, nostack));
    }
}

pub unsafe fn atomic<F, R>(f: F) -> R
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

    // Machine mode interrupts are disabled, execute the atomic
    // (uninterruptible) function
    let res = f();

    // If [`mstatus::mie`] was set before, set it again. Otherwise,
    // this function will be a nop.
    CSR.mstatus.read_and_set_bits(original_mie);

    res
}

// Mock implementations for tests on Travis-CI.
#[cfg(not(any(doc, target_os = "none")))]
/// NOP instruction (mock)
pub fn nop() {
    unimplemented!()
}

#[cfg(not(any(doc, target_os = "none")))]
/// WFI instruction (mock)
pub unsafe fn wfi() {
    unimplemented!()
}

// TODO: Cache ops for RISCV

pub unsafe fn prepare_dma(_range: NonNull<[u8]>) {}

pub unsafe fn finish_dma(_range: NonNull<[u8]>) {}

pub fn executable_memory_changed(_range: NonNull<[u8]>) {}
