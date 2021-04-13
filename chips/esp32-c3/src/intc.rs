//! Platform Level Interrupt Control peripheral driver.

use kernel::common::cells::VolatileCell;
use kernel::common::registers::interfaces::{Readable, Writeable};
use kernel::common::registers::{
    register_bitfields, register_structs, LocalRegisterCopy, ReadWrite,
};
use kernel::common::StaticRef;

register_structs! {
    pub IntcRegisters {
        (0x000 => _reserved0),
        (0x104 => enable: ReadWrite<u32, INT::Register>),
        (0x108 => type_reg: ReadWrite<u32, INT::Register>),
        (0x10C => clear: ReadWrite<u32, INT::Register>),
        (0x110 => eip: ReadWrite<u32, INT::Register>),
        (0x114 => _reserved1),
        (0x118 => priority: [ReadWrite<u32, PRIORITY::Register>; 31]),
        (0x194 => thresh: ReadWrite<u32, THRESH::Register>),
        (0x198 => @END),
    }
}

register_bitfields![u32,
    INT [
        ONE OFFSET(1) NUMBITS(1) [],
        TWO OFFSET(2) NUMBITS(1) [],
        THREE OFFSET(3) NUMBITS(1) [],
        FOUR OFFSET(4) NUMBITS(1) [],
        FIVE OFFSET(5) NUMBITS(1) [],
        SIX OFFSET(6) NUMBITS(1) [],
        SEVEN OFFSET(7) NUMBITS(1) [],
        EIGHT OFFSET(8) NUMBITS(1) [],
    ],
    PRIORITY [
        PRIORITY OFFSET(0) NUMBITS(4) [],
    ],
    THRESH [
        THRESH OFFSET(0) NUMBITS(4) [],
    ],
];

pub struct Intc {
    registers: StaticRef<IntcRegisters>,
    saved: VolatileCell<LocalRegisterCopy<u32>>,
}

impl Intc {
    pub const fn new(base: StaticRef<IntcRegisters>) -> Self {
        Intc {
            registers: base,
            saved: VolatileCell::new(LocalRegisterCopy::new(0)),
        }
    }

    /// Clear all pending interrupts.
    pub fn clear_all_pending(&self) {
        self.registers.clear.set(0xFF);
    }

    /// Enable all interrupts.
    pub fn enable_all(&self) {
        self.registers.enable.set(0xFF);

        // Set some default priority for each interrupt. This is not really used
        // at this point.
        for priority in self.registers.priority.iter() {
            priority.write(PRIORITY::PRIORITY.val(3));
        }

        // Accept all interrupts.
        self.registers.thresh.write(THRESH::THRESH.val(1));
    }

    /// Disable interrupt.
    pub fn disable(&self, irq: u32) {
        let mask = !(1 << irq);
        let value = self.registers.enable.get() & mask;
        self.registers.enable.set(value);
    }

    /// Disable all interrupts.
    pub fn disable_all(&self) {
        self.registers.enable.set(0x00);
    }

    /// Get the index (0-256) of the lowest number pending interrupt, or `None` if
    /// none is pending. RISC-V Intc has a "claim" register which makes it easy
    /// to grab the highest priority pending interrupt.
    pub fn next_pending(&self) -> Option<u32> {
        let eip = self.registers.eip.get();
        if eip == 0 {
            None
        } else {
            Some(eip.trailing_zeros())
        }
    }

    /// Save the current interrupt to be handled later
    /// This will save the interrupt at index internally to be handled later.
    /// Interrupts must be disabled before this is called.
    /// Saved interrupts can be retrieved by calling `get_saved_interrupts()`.
    /// Saved interrupts are cleared when `'complete()` is called.
    pub unsafe fn save_interrupt(&self, irq: u32) {
        // OR the current saved state with the new value
        let new_saved = self.saved.get().get() | 1 << irq;

        // Set the new state
        self.saved.set(LocalRegisterCopy::new(new_saved));
    }

    /// The `next_pending()` function will only return enabled interrupts.
    /// This function will return a pending interrupt that has been disabled by
    /// `save_interrupt()`.
    pub fn get_saved_interrupts(&self) -> Option<u32> {
        let saved = self.saved.get().get();
        if saved != 0 {
            return Some(saved.trailing_zeros());
        }

        None
    }

    /// Signal that an interrupt is finished being handled. In Tock, this should be
    /// called from the normal main loop (not the interrupt handler).
    /// Interrupts must be disabled before this is called.
    pub unsafe fn complete(&self, irq: u32) {
        // OR the current saved state with the new value
        let new_saved = self.saved.get().get() & !(1 << irq);

        // Set the new state
        self.saved.set(LocalRegisterCopy::new(new_saved));
    }
}
