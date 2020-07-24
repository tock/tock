//! Platform Level Interrupt Control peripheral driver.

use kernel::common::cells::VolatileCell;
use kernel::common::registers::LocalRegisterCopy;
use kernel::common::registers::{register_bitfields, ReadWrite};
use kernel::common::StaticRef;

pub const PLIC_BASE: StaticRef<PlicRegisters> =
    unsafe { StaticRef::new(0x0c00_0000 as *const PlicRegisters) };

#[repr(C)]
pub struct PlicRegisters {
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

pub struct Plic {
    registers: StaticRef<PlicRegisters>,
    saved: [VolatileCell<LocalRegisterCopy<u32>>; 2],
}

impl Plic {
    pub const fn new(base: StaticRef<PlicRegisters>) -> Self {
        Plic {
            registers: base,
            saved: [
                VolatileCell::new(LocalRegisterCopy::new(0)),
                VolatileCell::new(LocalRegisterCopy::new(0)),
            ],
        }
    }

    /// Clear all pending interrupts.
    pub fn clear_all_pending(&self) {
        for pending in self.registers.pending.iter() {
            pending.set(0);
        }
    }

    /// Enable all interrupts.
    pub fn enable_all(&self) {
        for enable in self.registers.enable.iter() {
            enable.set(0xFFFF_FFFF);
        }

        // Set some default priority for each interrupt. This is not really used
        // at this point.
        for priority in self.registers.priority.iter() {
            priority.write(priority::Priority.val(4));
        }

        // Accept all interrupts.
        self.registers.threshold.write(priority::Priority.val(0));
    }

    /// Disable all interrupts.
    pub fn disable_all(&self) {
        for enable in self.registers.enable.iter() {
            enable.set(0);
        }
    }

    /// Get the index (0-256) of the lowest number pending interrupt, or `None` if
    /// none is pending. RISC-V PLIC has a "claim" register which makes it easy
    /// to grab the highest priority pending interrupt.
    pub fn next_pending(&self) -> Option<u32> {
        let claim = self.registers.claim.get();
        if claim == 0 {
            None
        } else {
            Some(claim)
        }
    }

    /// Save the current interrupt to be handled later
    /// This will save the interrupt at index internally to be handled later.
    /// Interrupts must be disabled before this is called.
    /// Saved interrupts can be retrieved by calling `get_saved_interrupts()`.
    pub unsafe fn save_interrupt(&self, index: u32) {
        let offset = if index < 32 { 0 } else { 1 };
        let irq = index % 32;

        // OR the current saved state with the new value
        let new_saved = self.saved[offset].get().get() | 1 << irq;

        // Set the new state
        self.saved[offset].set(LocalRegisterCopy::new(new_saved));
    }

    /// The `next_pending()` function will only return enabled interrupts.
    /// This function will return a pending interrupt that has been disabled by
    /// `save_interrupt()`.
    pub fn get_saved_interrupts(&self) -> Option<u32> {
        for (i, pending) in self.saved.iter().enumerate() {
            let saved = pending.get().get();
            if saved != 0 {
                return Some(saved.trailing_zeros() + (i as u32 * 32));
            }
        }

        None
    }

    /// Signal that an interrupt is finished being handled. In Tock, this should be
    /// called from the normal main loop (not the interrupt handler).
    pub fn complete(&self, index: u32) {
        self.registers.claim.set(index);
    }

    /// Return `true` if there are any pending interrupts in the PLIC, `false`
    /// otherwise.
    pub fn has_pending(&self) -> bool {
        self.registers
            .pending
            .iter()
            .fold(0, |i, pending| pending.get() | i)
            != 0
    }

    /// This is a generic implementation. There may be board specific versions as
    /// some platforms have added more bits to the `mtvec` register.
    pub fn suppress_all(&self) {
        // Accept all interrupts.
        self.registers.threshold.write(priority::Priority.val(0));
    }
}
