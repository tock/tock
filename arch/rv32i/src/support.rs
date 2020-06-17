//! Core low-level operations.

use crate::csr::{mstatus::mstatus, CSR};
use core::ops::FnOnce;

#[cfg(all(target_arch = "riscv32", target_os = "none"))]
#[inline(always)]
/// NOP instruction
pub fn nop() {
    unsafe {
        llvm_asm!("nop" :::: "volatile");
    }
}

#[cfg(all(target_arch = "riscv32", target_os = "none"))]
#[inline(always)]
/// WFI instruction
pub unsafe fn wfi() {
    llvm_asm!("wfi" :::: "volatile");
}

pub unsafe fn atomic<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let original_mstatus = CSR.mstatus.extract();
    if original_mstatus.is_set(mstatus::mie) {
        CSR.mstatus
            .modify_no_read(original_mstatus, mstatus::mie::CLEAR);
    }
    let res = f();
    if original_mstatus.is_set(mstatus::mie) {
        CSR.mstatus
            .modify_no_read(original_mstatus, mstatus::mie::SET);
    }
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
