// TODO: This module may require renaming, as in VexRiscv the
// interrupt controller probably isn't really called "PLIC"

pub unsafe fn next_pending() -> Option<usize> {
    let pending_interrupts = vexriscv_irq_raw::irq_pending();

    // This is essentially an inefficient version of C's find first
    // set (ffs()) function, giving the index of the least significant
    // bit that is set
    (0..=31).find(|test| pending_interrupts & (1 << test) != 0)
}

/// Disable all interrupts
pub unsafe fn disable_interrupts() {
    vexriscv_irq_raw::irq_setie(false);
}

/// Enable all interrupts
pub unsafe fn enable_interrupts() {
    vexriscv_irq_raw::irq_setie(true);
}

/// Suppress (mask) a specific interrupt source
pub unsafe fn mask_interrupt(idx: usize) {
    vexriscv_irq_raw::irq_setmask(vexriscv_irq_raw::irq_getmask() & !(1 << idx));
}

/// Unsuppress (unmask) a specific interrupt source
pub unsafe fn unmask_interrupt(idx: usize) {
    vexriscv_irq_raw::irq_setmask(vexriscv_irq_raw::irq_getmask() | (1 << idx));
}

pub unsafe fn unmask_all_interrupts() {
    vexriscv_irq_raw::irq_setmask(0xFF);
}

pub unsafe fn mask_all_interrupts() {
    vexriscv_irq_raw::irq_setmask(0x00);
}

mod vexriscv_irq_raw {
    //! These functions mirror those of litex/soc/cores/vexriscv/irq.h
    //! which might be unsafe for direct use or behave unexpectedly
    //! and are hence wrapped in this private module
    #![allow(dead_code)]

    use rv32i::csr::{mstatus, CSR};

    /// defined in litex/soc/cores/cpu/vexriscv/csr-defs.h
    const CSR_IRQ_MASK: usize = 0xBC0;
    /// defined in litex/soc/cores/cpu/vexriscv/csr-defs.h
    const CSR_IRQ_PENDING: usize = 0xFC0;

    pub unsafe fn irq_getie() -> bool {
        CSR.mstatus.read(mstatus::mstatus::mie) != 0
    }

    pub unsafe fn irq_setie(ie: bool) {
        CSR.mstatus.modify(mstatus::mstatus::mie.val(ie as u32))
    }

    pub unsafe fn irq_getmask() -> usize {
        let mask: usize;
        asm!("csrr {mask}, {csr}", mask = out(reg) mask, csr = const CSR_IRQ_MASK);
        mask
    }

    pub unsafe fn irq_setmask(mask: usize) {
        asm!("csrw {csr}, {mask}", csr = const CSR_IRQ_MASK, mask = in(reg) mask);
    }

    pub unsafe fn irq_pending() -> usize {
        let pending: usize;
        asm!("csrr {pending}, {csr}", pending = out(reg) pending, csr = const CSR_IRQ_PENDING);
        pending
    }
}
