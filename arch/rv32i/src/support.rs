//! Core low-level operations.

use core::ops::FnOnce;

#[cfg(all(target_arch = "riscv32", target_os = "none"))]
#[inline(always)]
/// NOP instruction
pub fn nop() {
    unsafe {
        asm!("nop" :::: "volatile");
    }
}

#[cfg(all(target_arch = "riscv32", target_os = "none"))]
#[inline(always)]
/// WFI instruction
pub unsafe fn wfi() {
    asm!("wfi" :::: "volatile");
}

/// TODO: implement
pub unsafe fn atomic<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    f()
}

#[cfg(all(target_arch = "riscv32", target_os = "none"))]
#[lang = "eh_personality"]
pub extern "C" fn eh_personality() {}

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
