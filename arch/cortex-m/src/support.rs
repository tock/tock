// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Helper functions for the Cortex-M architecture.

use crate::scb;

/// NOP instruction
#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
#[inline(always)]
pub fn nop() {
    use core::arch::asm;

    // # Safety
    //
    // - INPUTS: This does not use the existing value of any registers.
    // - OUTPUTS: This does not write any registers.
    // - Options set:
    //   - nomem: We do not read or write memory.
    //   - nostack: This does not use the stack.
    //   - preserves_flags: This does not change flags.
    // - Options not set:
    //   - pure: not required
    //   - readonly: implied by nomem
    //   - noreturn: we do fall-through
    //   - att_syntax: not on arm
    //   - raw: not required
    unsafe {
        asm!("nop", options(nomem, nostack, preserves_flags));
    }
}

/// WFI instruction
#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
#[inline(always)]
pub unsafe fn wfi() {
    use core::arch::asm;

    // # Safety
    //
    // - INPUTS: This does not use the existing value of any registers.
    // - OUTPUTS: This does not write any registers.
    // - Options set:
    //   - nomem: We do not read or write memory.
    //   - nostack: This does not use the stack.
    //   - preserves_flags: This does not change flags.
    // - Options not set:
    //   - pure: not required
    //   - readonly: implied by nomem
    //   - noreturn: we do fall-through
    //   - att_syntax: not on arm
    //   - raw: not required
    unsafe {
        asm!("wfi", options(nomem, nostack, preserves_flags));
    }
}

/// Single-core critical section operation
#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
pub unsafe fn with_interrupts_disabled<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    use core::arch::asm;
    // Set PRIMASK to disable interrupts.
    //
    // # Safety
    //
    // - INPUTS: This does not use the existing value of any registers.
    // - OUTPUTS: This does not write any registers.
    // - Options set:
    //   - nomem: We do not read or write memory.
    //   - nostack: This does not use the stack.
    //   - preserves_flags: This does not change flags.
    // - Options not set:
    //   - pure: not required
    //   - readonly: implied by nomem
    //   - noreturn: we do fall-through
    //   - att_syntax: not on arm
    //   - raw: not required
    unsafe {
        asm!("cpsid i", options(nomem, nostack, preserves_flags));
    }

    let res = f();

    // Unset PRIMASK to re-enable interrupts.
    //
    // # Safety
    //
    // - INPUTS: This does not use the existing value of any registers.
    // - OUTPUTS: This does not write any registers.
    // - Options set:
    //   - nomem: We do not read or write memory.
    //   - nostack: This does not use the stack.
    //   - preserves_flags: This does not change flags.
    // - Options not set:
    //   - pure: not required
    //   - readonly: implied by nomem
    //   - noreturn: we do fall-through
    //   - att_syntax: not on arm
    //   - raw: not required
    unsafe {
        asm!("cpsie i", options(nomem, nostack, preserves_flags));
    }
    res
}

/// NOP instruction (mock)
// Mock implementations for tests on Travis-CI.
#[cfg(not(any(doc, all(target_arch = "arm", target_os = "none"))))]
pub fn nop() {
    unimplemented!()
}

/// WFI instruction (mock)
#[cfg(not(any(doc, all(target_arch = "arm", target_os = "none"))))]
pub unsafe fn wfi() {
    unimplemented!()
}

/// Single-core critical section operation (mock)
#[cfg(not(any(doc, all(target_arch = "arm", target_os = "none"))))]
pub unsafe fn with_interrupts_disabled<F, R>(_f: F) -> R
where
    F: FnOnce() -> R,
{
    unimplemented!()
}

/// Reset the chip.
pub fn reset() -> ! {
    unsafe {
        scb::reset();
    }
    loop {
        // This is required to avoid the empty loop clippy
        // warning #[warn(clippy::empty_loop)]
        nop();
    }
}

/// Check if we are executing in an interrupt handler or not.
///
/// Returns `true` if the CPU is executing in an interrupt handler. Returns
/// `false` if the chip is executing in thread mode.
#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
pub fn is_interrupt_context() -> bool {
    use core::arch::asm;
    let mut interrupt_number: u32;

    // # Safety
    //
    // - INPUTS: This does not use the existing value of any registers.
    // - OUTPUTS: This writes `r0` which is specified as an output.
    // - Options set:
    //   - nomem: We do not read or write memory.
    //   - nostack: This does not use the stack.
    //   - preserves_flags: This does not change flags.
    // - Options not set:
    //   - pure: not required
    //   - readonly: implied by nomem
    //   - noreturn: we do fall-through
    //   - att_syntax: not on arm
    //   - raw: not required
    unsafe {
        // IPSR[8:0] holds the currently active interrupt
        asm!(
            "
    mrs r0, ipsr
            ",
            out("r0") interrupt_number,
            options(nomem, nostack, preserves_flags)
        );
    }

    // If IPSR[8:0] is 0 then we are in thread mode. Otherwise an interrupt has
    // occurred and we are in some interrupt service routine.
    (interrupt_number & 0x1FF) != 0
}

#[cfg(not(any(doc, all(target_arch = "arm", target_os = "none"))))]
pub fn is_interrupt_context() -> bool {
    unimplemented!()
}
