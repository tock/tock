// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Platform Level Interrupt Control peripheral driver.

use kernel::utilities::cells::VolatileCell;
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::LocalRegisterCopy;
use kernel::utilities::registers::{register_bitfields, ReadOnly, ReadWrite};
use kernel::utilities::StaticRef;

#[repr(C)]
pub struct PlicRegisters {
    /// Interrupt Priority Register
    _reserved0: u32,
    priority: [ReadWrite<u32, priority::Register>; 51],
    _reserved1: [u8; 3888],
    /// Interrupt Pending Register
    pending: [ReadOnly<u32>; 2],
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

    /// Clear all pending interrupts. The [`E31 core manual`] PLIC Chapter 9.8
    /// p 117: A successful claim also atomically clears the corresponding
    /// pending bit on the interrupt source.
    /// Note that this function requires you call `enable_all()` first! (As ch.
    /// 9.4 p.114 writes.)
    ///
    /// [`E31 core manual`]: https://sifive.cdn.prismic.io/sifive/c29f9c69-5254-4f9a-9e18-24ea73f34e81_e31_core_complex_manual_21G2.pdf
    pub fn clear_all_pending(&self) {
        let regs = self.registers;

        loop {
            let id = regs.claim.get();
            if id == 0 {
                break;
            }
            regs.claim.set(id);
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
    /// Saved interrupts are cleared when `'complete()` is called.
    pub unsafe fn save_interrupt(&self, index: u32) {
        let offset = usize::from(index >= 32);
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

        let offset = usize::from(index >= 32);
        let irq = index % 32;

        // OR the current saved state with the new value
        let new_saved = self.saved[offset].get().get() & !(1 << irq);

        // Set the new state
        self.saved[offset].set(LocalRegisterCopy::new(new_saved));
    }

    /// This is a generic implementation. There may be board specific versions as
    /// some platforms have added more bits to the `mtvec` register.
    pub fn suppress_all(&self) {
        // Accept all interrupts.
        self.registers.threshold.write(priority::Priority.val(0));
    }
}
