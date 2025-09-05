// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! RISC-V support for getting the running thread ID.

use kernel::platform::chip::ThreadIdProvider;

/// Implement [`ThreadIdProvider`] for RISC-V.
pub enum RiscvThreadIdProvider {}

// # Safety
//
// By implementing [`ThreadIdProvider`] we are guaranteeing that we correctly
// return the thread ID. On single-core platforms the thread ID only depends on
// whether execution is in an interrupt service routine or not, which is what
// this implementation checks for.
unsafe impl ThreadIdProvider for RiscvThreadIdProvider {
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
