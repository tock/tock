// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! VexRiscv-specific interrupt controller implementation

use core::cell::Cell;

/// Rust wrapper around the raw CSR-based VexRiscv interrupt
/// controller
///
/// The wrapper supports saving all currently pending interrupts to an
/// internal state, which can then be used for interrupt processing.
pub struct VexRiscvInterruptController {
    saved_interrupts: Cell<usize>,
}

impl VexRiscvInterruptController {
    /// Construct a new VexRiscvInterruptController instance
    pub const fn new() -> Self {
        VexRiscvInterruptController {
            saved_interrupts: Cell::new(0),
        }
    }

    /// Save the currently pending interrupts in hardware to the
    /// internal state
    ///
    /// This should be accessed in an atomic context to ensure a
    /// consistent view on the pending interrupts is saved.
    pub unsafe fn save_pending(&self) -> bool {
        let all_pending = vexriscv_irq_raw::irq_pending();
        self.saved_interrupts.set(all_pending);

        // return true if some interrupts were saved
        all_pending != 0
    }

    /// Return the next pending interrupts in the saved state
    ///
    /// If no interrupt is pending in the saved state, this function
    /// returns `None`.
    ///
    /// The ordering is determined by the interrupt number, lower
    /// having a higher priority.
    pub fn next_saved(&self) -> Option<usize> {
        let saved_interrupts: usize = self.saved_interrupts.get();
        let interrupt_bits = usize::BITS as usize;

        // If there are no interrupts pending (saved_interrupts == 0),
        // usize::trailing_zeros will return usize::BITS, in which
        // case we need to return None
        let trailing_zeros = usize::trailing_zeros(saved_interrupts) as usize;
        if trailing_zeros == interrupt_bits {
            None
        } else {
            Some(trailing_zeros)
        }
    }

    /// Mark a saved interrupt as complete, removing it from the
    /// `next_saved` queue
    ///
    /// If all interrupts are marked as complete, `next_saved` will
    /// return `None`.
    pub fn complete_saved(&self, idx: usize) {
        self.saved_interrupts
            .set(self.saved_interrupts.get() & !(1 << idx));
    }

    /// Suppress (mask) a specific interrupt source in the interrupt
    /// controller
    pub unsafe fn mask_interrupt(idx: usize) {
        vexriscv_irq_raw::irq_setmask(vexriscv_irq_raw::irq_getmask() & !(1 << idx));
    }

    /// Unsuppress (unmask) a specific interrupt source in the
    /// interrupt controller
    pub unsafe fn unmask_interrupt(idx: usize) {
        vexriscv_irq_raw::irq_setmask(vexriscv_irq_raw::irq_getmask() | (1 << idx));
    }

    /// Suppress (mask) all interrupts in the interrupt controller
    pub unsafe fn mask_all_interrupts() {
        vexriscv_irq_raw::irq_setmask(0);
    }

    /// Unsuppress (unmask) all interrupts in the interrupt controller
    pub unsafe fn unmask_all_interrupts() {
        vexriscv_irq_raw::irq_setmask(usize::MAX);
    }
}

mod vexriscv_irq_raw {
    //! These functions mirror those of litex/soc/cores/vexriscv/irq.h
    //! which might be unsafe for direct use or behave unexpectedly
    //! and are hence wrapped in this private module
    #![allow(dead_code)]

    /// defined in litex/soc/cores/cpu/vexriscv/csr-defs.h
    const CSR_IRQ_MASK: usize = 0xBC0;
    /// defined in litex/soc/cores/cpu/vexriscv/csr-defs.h
    const CSR_IRQ_PENDING: usize = 0xFC0;

    #[cfg(not(any(doc, all(target_arch = "riscv32", target_os = "none"))))]
    pub unsafe fn irq_getmask() -> usize {
        0
    }

    #[cfg(any(doc, all(target_arch = "riscv32", target_os = "none")))]
    pub unsafe fn irq_getmask() -> usize {
        let mask: usize;
        use core::arch::asm;
        asm!("csrr {mask}, {csr}", mask = out(reg) mask, csr = const CSR_IRQ_MASK);
        mask
    }

    #[cfg(not(any(doc, all(target_arch = "riscv32", target_os = "none"))))]
    pub unsafe fn irq_setmask(_mask: usize) {}

    #[cfg(any(doc, all(target_arch = "riscv32", target_os = "none")))]
    pub unsafe fn irq_setmask(mask: usize) {
        use core::arch::asm;
        asm!("csrw {csr}, {mask}", csr = const CSR_IRQ_MASK, mask = in(reg) mask);
    }

    #[cfg(not(any(doc, all(target_arch = "riscv32", target_os = "none"))))]
    pub unsafe fn irq_pending() -> usize {
        0
    }

    #[cfg(any(doc, all(target_arch = "riscv32", target_os = "none")))]
    pub unsafe fn irq_pending() -> usize {
        let pending: usize;
        use core::arch::asm;
        asm!("csrr {pending}, {csr}", pending = out(reg) pending, csr = const CSR_IRQ_PENDING);
        pending
    }
}
