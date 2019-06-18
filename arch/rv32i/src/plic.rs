//! Platform Level Interrupt Control

use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite};
use kernel::common::StaticRef;

#[repr(C)]
struct PlicRegisters {
    /// Interrupt Priority Register
    _reserved0: u32,
    priority: [ReadWrite<u32, priority::Register>; (0x0C00_00D0 - 0x0C00_0004) / 0x4],
    _reserved1: [u32; (0x0C00_1000 - 0x0C00_00D0) / 0x4],
    /// Interrupt Pending Register
    pending: [ReadOnly<u32>; (0x0C00_1008 - 0x0C00_1000) / 0x4],
    _reserved2: [u32; (0x0C00_2000 - 0x0C00_1008) / 0x4],
    /// Interrupt Enable Register
    enable: [ReadWrite<u32>; 2],
    _reserved3: [u32; (0x0C20_0000 - 0x0C00_2008) / 0x4],
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
    loop {
        let id_wrapper = next_pending();
        match id_wrapper {
            None => break,
            Some(id) => complete(id),
        }
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

pub unsafe fn surpress_all() {
    let plic: &PlicRegisters = &*PLIC_BASE;
    // Accept all interrupts.
    plic.threshold.write(priority::Priority.val(0));
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

    plic.pending.iter().fold(0, |i, pending| pending.get() | i) != 0
}
