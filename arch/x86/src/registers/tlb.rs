// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

// This is inspired and adapted for Tock from the [x86](https://github.com/gz/rust-x86) crate.

//! Functions to flush the translation lookaside buffer (TLB).

/// Invalidate the given address in the TLB using the `invlpg` instruction.
///
/// # Safety
/// This function is unsafe as it causes a general protection fault (GP) if the current privilege
/// level is not 0.
#[cfg(target_arch = "x86")]
pub unsafe fn flush(addr: usize) {
    use core::arch::asm;

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
    use crate::registers::controlregs;
    unsafe { controlregs::cr3_write(controlregs::cr3()) }
}

// For CI only

#[cfg(not(any(doc, target_arch = "x86")))]
pub unsafe fn flush(_addr: usize) {
    unimplemented!()
}
