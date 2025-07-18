// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Core low-level operations.

use crate::csr::{mstatus::mstatus, CSR};

#[cfg(any(doc, all(target_arch = "riscv32", target_os = "none")))]
#[inline(always)]
/// NOP instruction
pub fn nop() {
    use core::arch::asm;
    unsafe {
        asm!("nop", options(nomem, nostack, preserves_flags));
    }
}

#[cfg(any(doc, all(target_arch = "riscv32", target_os = "none")))]
#[inline(always)]
/// WFI instruction
pub unsafe fn wfi() {
    use core::arch::asm;
    asm!("wfi", options(nomem, nostack));
}

/// Single-core critical section operation
pub unsafe fn with_interrupts_disabled<F, R>(f: F) -> R
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
#[cfg(not(any(doc, all(target_arch = "riscv32", target_os = "none"))))]
/// NOP instruction (mock)
pub fn nop() {
    unimplemented!()
}

#[cfg(not(any(doc, all(target_arch = "riscv32", target_os = "none"))))]
/// WFI instruction (mock)
pub unsafe fn wfi() {
    unimplemented!()
}

pub enum RiscvThreadIdProvider {}
unsafe impl kernel::platform::chip::ThreadIdProvider for RiscvThreadIdProvider {
    /// Return the current thread ID, computed using the `mhartid` (hardware thread
    /// ID), and a flag indicating whether the current hart is currently in a trap
    /// handler context.
    fn running_thread_id() -> usize {
        // Mock implementation for non-rv32 target builds:
        #[cfg(not(all(target_arch = "riscv32", target_os = "none")))]
        {
            unimplemented!()
        }

        // Proper rv32i-specific implementation:
        #[cfg(all(target_arch = "riscv32", target_os = "none"))]
        {
            let hartid: usize;
            let trap_handler_active_addr: *mut usize;

            // Safety:
            //
            // This does not read any memory by itself, it merely loads a symbol
            // address and reads the `mhartid` CSR:
            unsafe {
                core::arch::asm!(
                    // Determine the hart id and save in the appropriate output
                    // register:
                    "csrr {hartid_reg}, mhartid",
                    // Load the `_trap_handler_active` symbol address:
                    "la {trap_handler_active_addr_reg}, _trap_handler_active",
                    hartid_reg = out(reg) hartid,
                    trap_handler_active_addr_reg = out(reg) trap_handler_active_addr,
                    options(
                    // The assembly code has no side effects, must eventually
                    // return, and its outputs depend only on its direct inputs
                    // (i.e. the values themselves, not what they point to) or
                    // values read from memory (unless the nomem options is also
                    // set).
                    pure,
                    // The assembly code does not read from or write to any memory
                    // accessible outside of the assembly code.
                    nomem,
                    // The assembly code does not modify the flags register (for
                    // RISC-V: `fflags`, `vtype`, `vl`, or `vcsr`).
                    preserves_flags,
                    // The assembly code does not push data to the stack, or write
                    // to the stack red-zone (if supported by the target).
                    nostack,
                    ),
                );
            }

            // Load the hart's trap_handler_active value.
            //
            // Safety:
            //
            // `hartid` * core::mem::size_of::<usize>() must fit into an `isize`. By
            // allocating the `_trap_handler_active` array, the chip crate ensures
            // that this array can fit all hart IDs for all harts on the target
            // machine. The maximum size of any Rust allocation is `isize::MAX`
            // bytes, and as such, by allocating this arrary in the first place, the
            // chip crate maintains that indexing into it with `hartid` must stay in
            // this object, and an index of `hartid` elements will not produce an
            // offset larger than `isize::MAX` bytes.
            let hart_trap_handler_active =
                unsafe { core::ptr::read(trap_handler_active_addr.add(hartid)) };

            // Determine the thread ID as a combination of the hart id, and whether
            // it is currently in a trap handler context:
            hartid.overflowing_shl(1).0 | ((hart_trap_handler_active != 0) as usize)
        }
    }
}
