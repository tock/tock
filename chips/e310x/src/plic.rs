//! Platform Level Interrupt Control peripheral driver.

use kernel::common::registers::{register_bitfields, ReadWrite};
use kernel::common::StaticRef;

#[repr(C)]
struct PlicRegisters {
    /// Interrupt Priority Register
    _reserved0: u32,
    priority: [ReadWrite<u32, priority::Register>; 51],
    _reserved1: [u8; 3888],
    /// Interrupt Pending Register
    pending: [ReadWrite<u32>; 2],
    _reserved2: [u8; 4088],
    /// Interrupt Enable Register
    enable: [ReadWrite<u32>; 2],
    _reserved3: [u8; 2088952],
    /// Priority Threshold Register
    threshold: ReadWrite<u32, priority::Register>,
    /// Claim/Complete Register
    claim: ReadWrite<u32>,
}

register_bitfields![u32,
    priority [
        Priority OFFSET(0) NUMBITS(3) []
    ]
];

const PLIC_BASE: StaticRef<PlicRegisters> =
    unsafe { StaticRef::new(0x0c00_0000 as *const PlicRegisters) };

/// Clear all pending interrupts.
pub unsafe fn clear_all_pending() {
    let plic: &PlicRegisters = &*PLIC_BASE;
    for pending in plic.pending.iter() {
        pending.set(0);
    }
}

/// Enable all interrupts.
pub unsafe fn enable_all() {
    let plic: &PlicRegisters = &*PLIC_BASE;
    for enable in plic.enable.iter() {
        enable.set(0xFFFF_FFFF);
    }

    // Set some default priority for each interrupt. This is not really used
    // at this point.
    for priority in plic.priority.iter() {
        priority.write(priority::Priority.val(4));
    }

    // Accept all interrupts.
    plic.threshold.write(priority::Priority.val(0));
}

/// Disable all interrupts.
pub unsafe fn disable_all() {
    let plic: &PlicRegisters = &*PLIC_BASE;
    for enable in plic.enable.iter() {
        enable.set(0);
    }
}

/// Get the index (0-256) of the lowest number pending interrupt, or `None` if
/// none is pending. RISC-V PLIC has a "claim" register which makes it easy
/// to grab the highest priority pending interrupt.
pub unsafe fn next_pending() -> Option<u32> {
    let plic: &PlicRegisters = &*PLIC_BASE;

    let claim = plic.claim.get();
    if claim == 0 {
        None
    } else {
        Some(claim)
    }
}

/// Signal that an interrupt is finished being handled. In Tock, this should be
/// called from the normal main loop (not the interrupt handler).
pub unsafe fn complete(index: u32) {
    let plic: &PlicRegisters = &*PLIC_BASE;
    plic.claim.set(index);
}

/// Return `true` if there are any pending interrupts in the PLIC, `false`
/// otherwise.
pub unsafe fn has_pending() -> bool {
    let plic: &PlicRegisters = &*PLIC_BASE;

    for (i, pending) in plic.pending.iter().enumerate() {
        if pending.get() & plic.enable[i].get() != 0 {
            return true;
        }
    }

    false
}

/// This is a generic implementation. There may be board specific versions as
/// some platforms have added more bits to the `mtvec` register.
pub unsafe fn suppress_all() {
    let plic: &PlicRegisters = &*PLIC_BASE;
    // Accept all interrupts.
    plic.threshold.write(priority::Priority.val(0));
}
