//! Core low-level operations.

use core::ops::FnOnce;

#[inline(always)]
/// NOP instruction
pub fn nop() {
    unsafe {
        asm!("nop" :::: "volatile");
    }
}

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

#[cfg(target_os = "none")]
#[lang = "eh_personality"]
pub extern "C" fn eh_personality() {}
