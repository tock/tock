// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Miscellaneous low-level operations

use core::arch::asm;

/// Execute a given closure atomically.
///
/// This function ensures interrupts are disabled before invoking the given closue `f`. This allows
/// you to safely perform memory accesses which would otherwise race against interrupt handlers.
#[cfg(target_arch = "x86")]
pub fn atomic<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    use x86::bits32::eflags::{self, EFlags};
    use x86::irq;

    // Safety: We assume that this function is only ever called from inside the Tock kernel itself
    //         running with a CPL of 0. This allows us to read EFLAGS and disable/enable interrupts
    //         without fear of triggering an exception.
    unsafe {
        let eflags = eflags::read();
        let enabled = eflags.contains(EFlags::FLAGS_IF);

        if enabled {
            irq::disable();
        }

        let res = f();

        if enabled {
            irq::enable();
        }

        res
    }
}

#[cfg(not(target_arch = "x86"))]
pub fn atomic<F, R>(_: F) -> R
where
    F: FnOnce() -> R,
{
    unimplemented!()
}

/// Executes a single NOP instruction.
#[inline(always)]
pub fn nop() {
    unsafe {
        asm!("nop", options(nomem, nostack, preserves_flags));
    }
}
