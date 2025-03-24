//! Functions to flush the translation lookaside buffer (TLB).

use core::arch::asm;

/// Invalidate the given address in the TLB using the `invlpg` instruction.
///
/// # Safety
/// This function is unsafe as it causes a general protection fault (GP) if the current privilege
/// level is not 0.
pub unsafe fn flush(addr: usize) {
    unsafe {
        asm!("invlpg ({})", in(reg) addr, options(att_syntax, nostack, preserves_flags));
    }
}

/// Invalidate the TLB completely by reloading the CR3 register.
///
/// # Safety
/// This function is unsafe as it causes a general protection fault (GP) if the current privilege
/// level is not 0.
pub unsafe fn flush_all() {
    use crate::tock_x86::controlregs;
    unsafe { controlregs::cr3_write(controlregs::cr3()) }
}
