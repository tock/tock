//! Platform Level Interrupt Control peripheral driver.

use kernel::common::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::common::StaticRef;
use kernel::debug;

register_structs! {
    pub PlicRegisters {
        /// Interrupt Pending Register
        (0x000 => pending: [ReadOnly<u32>; 3]),
        /// Interrupt Source Register
        (0x00C => source: [ReadWrite<u32>; 3]),
        /// Interrupt Priority Registers
        (0x018 => priority: [ReadWrite<u32, priority::Register>; 79]),
        (0x154 => _reserved0: [ReadWrite<u32>; 43]),
        /// Interrupt Enable Register
        (0x200 => enable: [ReadWrite<u32>; 3]),
        /// Priority Threshold Register
        (0x20C => threshold: ReadWrite<u32, priority::Register>),
        /// Claim/Complete Register
        (0x210 => claim: ReadWrite<u32>),
        /// MSIP Register
        (0x214 => msip: ReadWrite<u32>),
        (0x218 => @END),
    }
}

register_bitfields![u32,
    priority [
        Priority OFFSET(0) NUMBITS(3) []
    ]
];

const PLIC_BASE: StaticRef<PlicRegisters> =
    unsafe { StaticRef::new(0x4009_0000 as *const PlicRegisters) };

/// Clear all pending interrupts.
pub unsafe fn clear_all_pending() {
    let _plic: &PlicRegisters = &*PLIC_BASE;
}

/// Enable all interrupts.
pub unsafe fn enable_all() {
    let plic: &PlicRegisters = &*PLIC_BASE;
    for enable in plic.enable.iter() {
	// DANGER: For some reason USBDEV seems to be issuing
	// unhandled CONNECT interrupts. So I have disabled them
	// here. This should be changed back to 0xFFFFFFFF once
	// this is resolved and before merging. -pal 8/26/20
        enable.set(0x0000_00FF);
    }

    // Set the max priority for each interrupt. This is not really used
    // at this point.
    for priority in plic.priority.iter() {
        priority.write(priority::Priority.val(3));
    }

    // Accept all interrupts.
    plic.threshold.write(priority::Priority.val(1));
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

    plic.pending.iter().fold(0, |i, pending| pending.get() | i) != 0
}
