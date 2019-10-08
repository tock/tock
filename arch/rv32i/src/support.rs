//! Core low-level operations.

use core::ops::FnOnce;

#[cfg(target_os = "none")]
#[inline(always)]
/// NOP instruction
pub fn nop() {
    unsafe {
        asm!("nop" :::: "volatile");
    }
}

#[cfg(not(target_os = "none"))]
/// NOP instruction (mock)
pub fn nop() {}

#[cfg(target_os = "none")]
#[inline(always)]
/// WFI instruction
pub unsafe fn wfi() {
    asm!("wfi" :::: "volatile");
}

#[cfg(not(target_os = "none"))]
/// WFI instruction (mock)
pub unsafe fn wfi() {}

/// TODO: implement
pub unsafe fn atomic<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    f()
}

#[cfg(target_os = "none")]
#[lang = "eh_personality"]
pub extern "C" fn eh_personality() {}
