// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Platform Level Interrupt Control peripheral driver.

use crate::registers::top_earlgrey::RV_PLIC_BASE_ADDR;
use kernel::utilities::cells::VolatileCell;
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::LocalRegisterCopy;
use kernel::utilities::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::utilities::StaticRef;

pub const PLIC_BASE: StaticRef<PlicRegisters> =
    unsafe { StaticRef::new(RV_PLIC_BASE_ADDR as *const PlicRegisters) };

pub static mut PLIC: Plic = Plic::new(PLIC_BASE);

pub const PLIC_REGS: usize = 6;

register_structs! {
    pub PlicRegisters {
        /// Interrupt Priority Registers
        (0x000 => priority: [ReadWrite<u32, priority::Register>; 181]),
        (0x2d4 => _reserved0),
        /// Interrupt Pending Register
        (0x1000 => pending: [ReadOnly<u32>; PLIC_REGS]),
        (0x1018 => _reserved1),
        /// Interrupt Enable Register
        (0x2000 => enable: [ReadWrite<u32>; PLIC_REGS]),
        (0x2018 => _reserved2),
        /// Priority Threshold Register
        (0x200000 => threshold: ReadWrite<u32, priority::Register>),
        /// Claim/Complete Register
        (0x200004 => claim: ReadWrite<u32>),
        (0x200008 => _reserved3),
        /// MSIP Register
        (0x4000000 => msip: ReadWrite<u32>),
        (0x4000004 => _reserved4),
        (0x4004000 => alert_test: ReadWrite<u32>),
        (0x4004004 => @END),
    }
}

register_bitfields![u32,
    priority [
        Priority OFFSET(0) NUMBITS(3) []
    ]
];

pub struct Plic {
    registers: StaticRef<PlicRegisters>,
    saved: [VolatileCell<LocalRegisterCopy<u32>>; PLIC_REGS],
}

impl Plic {
    pub const fn new(base: StaticRef<PlicRegisters>) -> Self {
        Plic {
            registers: base,
            saved: [
                VolatileCell::new(LocalRegisterCopy::new(0)),
                VolatileCell::new(LocalRegisterCopy::new(0)),
                VolatileCell::new(LocalRegisterCopy::new(0)),
                VolatileCell::new(LocalRegisterCopy::new(0)),
                VolatileCell::new(LocalRegisterCopy::new(0)),
                VolatileCell::new(LocalRegisterCopy::new(0)),
            ],
        }
    }

    /// Clear all pending interrupts.
    pub fn clear_all_pending(&self) {
        unimplemented!()
    }

    /// Enable all interrupts.
    pub fn enable_all(&self) {
        for enable in self.registers.enable.iter() {
            enable.set(0xFFFF_FFFF);
        }

        // Set the max priority for each interrupt. This is not really used
        // at this point.
        for priority in self.registers.priority.iter() {
            priority.write(priority::Priority.val(3));
        }

        // Accept all interrupts.
        self.registers.threshold.write(priority::Priority.val(1));
    }

    /// Disable specific interrupt.
    pub fn disable(&self, index: u32) {
        let offset = if index < 32 {
            0
        } else if index < 64 {
            1
        } else if index < 96 {
            2
        } else if index < 128 {
            3
        } else if index < 160 {
            4
        } else if index < 192 {
            5
        } else {
            panic!("Invalid IRQ: {}", index);
        };

        let irq = index % 32;
        let mask = !(1 << irq);

        self.registers.enable[offset].set(self.registers.enable[offset].get() & mask);
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
    /// Saved interrupts are cleared when `'complete()` is called.
    pub unsafe fn save_interrupt(&self, index: u32) {
        let offset = if index < 32 {
            0
        } else if index < 64 {
            1
        } else if index < 96 {
            2
        } else if index < 128 {
            3
        } else if index < 160 {
            4
        } else if index < 192 {
            5
        } else {
            panic!("Invalid IRQ: {}", index);
        };
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
    /// Interrupts must be disabled before this is called.
    pub unsafe fn complete(&self, index: u32) {
        self.registers.claim.set(index);

        let offset = if index < 32 {
            0
        } else if index < 64 {
            1
        } else if index < 96 {
            2
        } else if index < 128 {
            3
        } else if index < 160 {
            4
        } else if index < 192 {
            5
        } else {
            panic!("Invalid IRQ: {}", index);
        };
        let irq = index % 32;

        // OR the current saved state with the new value
        let new_saved = self.saved[offset].get().get() & !(1 << irq);

        // Set the new state
        self.saved[offset].set(LocalRegisterCopy::new(new_saved));
    }
}
