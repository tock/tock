// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Core low-level operations.

use crate::csr::{CSR, mstatus::mstatus};

#[cfg(riscv)]
#[inline(always)]
/// NOP instruction
pub fn nop() {
    use core::arch::asm;

    // SAFETY: This complies with the asm safety requirements:
    // - INPUTS: This does not use the existing value of any registers.
    // - OUTPUTS: This does not write any registers.
    // - Options set:
    //   - nomem: We do not read or write memory.
    //   - nostack: This does not use the stack.
    //   - preserves_flags: This does not change flags.
    // - Options not set:
    //   - preserves_flags: no meaning on RISC-V
    //   - pure: not required
    //   - readonly: implied by nomem
    //   - noreturn: we do fall-through
    //   - att_syntax: not on riscv
    //   - raw: not required
    unsafe {
        asm!("nop", options(nomem, nostack));
    }
}

#[cfg(riscv)]
#[inline(always)]
/// Wait For Interrupt (WFI) instruction.
pub unsafe fn wfi() {
    use core::arch::asm;

    // SAFETY: This complies with the asm safety requirements:
    // - INPUTS: This does not use the existing value of any registers.
    // - OUTPUTS: This does not write any registers.
    // - Options set:
    //   - nomem: We do not read or write memory.
    //   - nostack: This does not use the stack.
    // - Options not set:
    //   - preserves_flags: no meaning on RISC-V
    //   - pure: not required
    //   - readonly: implied by nomem
    //   - noreturn: we do fall-through
    //   - att_syntax: not on riscv
    //   - raw: not required
    unsafe {
        asm!("wfi", options(nomem, nostack));
    }
}

/// Single-core critical section operation
pub fn with_interrupts_disabled<F, R>(f: F) -> R
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

    // Machine mode interrupts are disabled, execute the (uninterruptible)
    // function
    let res = f();

    // If [`mstatus::mie`] was set before, set it again. Otherwise,
    // this function will be a nop.
    CSR.mstatus.read_and_set_bits(original_mie);

    res
}

// Mock implementations for tests on Travis-CI.
#[cfg(not(riscv))]
pub fn nop() {
    unimplemented!()
}

#[cfg(not(riscv))]
pub unsafe fn wfi() {
    unimplemented!()
}
