//! Core low-level operations.

use crate::csr::{mstatus::mstatus, CSR};
use core::ops::FnOnce;

#[cfg(all(target_arch = "riscv32", target_os = "none"))]
#[inline(always)]
/// NOP instruction
pub fn nop() {
    unsafe {
        asm!("nop", options(nomem, nostack, preserves_flags));
    }
}

#[cfg(all(target_arch = "riscv32", target_os = "none"))]
#[inline(always)]
/// WFI instruction
pub unsafe fn wfi() {
    asm!("wfi", options(nomem, nostack));
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
#[cfg(not(any(target_arch = "riscv32", target_os = "none")))]
/// NOP instruction (mock)
pub fn nop() {
    unimplemented!()
}

#[cfg(not(any(target_arch = "riscv32", target_os = "none")))]
/// WFI instruction (mock)
pub unsafe fn wfi() {
    unimplemented!()
}
