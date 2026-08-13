// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Miscellaneous low-level operations

#[cfg(any(doc, target_arch = "x86"))]
use core::arch::asm;
use kernel::platform::interrupts_disabled::InterruptsDisabled;

/// Execute closure without allowing any interrupts on the current core.
///
/// This function ensures interrupts are disabled before invoking the given
/// closue `f`. This allows / you to safely perform single-core operations
/// which would otherwise race against interrupt handlers.
#[cfg(any(doc, target_arch = "x86"))]
pub fn with_interrupts_disabled<F, R>(f: F) -> R
where
    F: FnOnce(&InterruptsDisabled) -> R,
{
    use crate::registers::bits32::eflags::{self, EFLAGS};
    use crate::registers::irq;

    // Safety: We assume that this function is only ever called from inside the Tock kernel itself
    //         running with a CPL of 0. This allows us to read EFLAGS and disable/enable interrupts
    //         without fear of triggering an exception.
    unsafe {
        let eflags = eflags::read();
        let enabled = eflags.0.is_set(EFLAGS::FLAGS_IF);

        if enabled {
            irq::disable();
        }

        // Safety: interrupts are disabled at this point, either because we
        // just disabled them above, or because they were already disabled
        // when this function was called.
        let interrupts_disabled = InterruptsDisabled::new_trusted();

        let res = f(&interrupts_disabled);

        if enabled {
            irq::enable();
        }

        res
    }
}

#[cfg(not(any(doc, target_arch = "x86")))]
pub fn with_interrupts_disabled<F, R>(_: F) -> R
where
    F: FnOnce(&InterruptsDisabled) -> R,
{
    unimplemented!()
}

/// Executes a single NOP instruction.
#[cfg(any(doc, target_arch = "x86"))]
#[inline(always)]
pub fn nop() {
    unsafe {
        asm!("nop", options(nomem, nostack, preserves_flags));
    }
}

#[cfg(not(any(doc, target_arch = "x86")))]
#[inline(always)]
pub fn nop() {
    unimplemented!()
}
