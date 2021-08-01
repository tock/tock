//! Platform Level Interrupt Control peripheral driver.

use crate::interrupts;
use kernel::utilities::cells::VolatileCell;
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{
    register_bitfields, register_structs, LocalRegisterCopy, ReadWrite,
};
use kernel::utilities::StaticRef;

register_structs! {
    pub IntcRegisters {
        (0x000 => _reserved0),
        (0x040 => gpio_interrupt_pro_map: ReadWrite<u32>),
        (0x044 => gpio_interrupt_pro_nmi_map: ReadWrite<u32>),
        (0x048 => _reserved1),
        (0x054 => uart0_intr_map: ReadWrite<u32>),
        (0x058 => _reserved2),
        (0x0f8 => status: [ReadWrite<u32>; 2]),
        (0x100 => clk_en: ReadWrite<u32>),
        (0x104 => enable: ReadWrite<u32, INT::Register>),
        (0x108 => type_reg: ReadWrite<u32, INT::Register>),
        (0x10C => clear: ReadWrite<u32, INT::Register>),
        (0x110 => eip: ReadWrite<u32, INT::Register>),
        (0x114 => _reserved3),
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

    /// The ESP32C3 is interesting. It allows interrupts to be mapped on the
    /// fly by setting the `intr_map` registers. This feature is completely
    /// undocumented. The ESP32 HAL and projects that use that (like Zephyr)
    /// call into the ROM code to enable interrupts which maps the interrupts.
    /// In Tock we map them ourselves so we don't need to call into the ROM.
    pub fn map_interrupts(&self) {
        self.registers.uart0_intr_map.set(interrupts::IRQ_UART0);
        self.registers
            .gpio_interrupt_pro_map
            .set(interrupts::IRQ_GPIO);
        self.registers
            .gpio_interrupt_pro_nmi_map
            .set(interrupts::IRQ_GPIO_NMI);
    }

    /// Clear all pending interrupts.
    pub fn clear_all_pending(&self) {
        self.registers.clear.set(0xFF);
    }

    /// Enable all interrupts.
    pub fn enable_all(&self) {
        self.registers.enable.set(0xFFFF_FFFF);

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
