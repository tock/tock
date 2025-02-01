// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Platform Level Interrupt Control peripheral driver.

use kernel::utilities::cells::VolatileCell;
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::LocalRegisterCopy;
use kernel::utilities::registers::{register_bitfields, ReadOnly, ReadWrite};
use kernel::utilities::StaticRef;

/// Place the register map definition in a private module to disallow direct access to it's
/// fields from the Plic struct implementation, which should only use a getter/setter with
/// appropriate bounds set

///    The generic SiFive PLIC specification:
///    <https://github.com/riscv/riscv-plic-spec/blob/master/riscv-plic.adoc>
///    is defining maximum of 1023 interrupt sources

// TODO: replace with const generic for `priority` and `_reserved1` field
// in the [PlicRegisters] when const generic expressions are stable
const MAX_INTERRUPTS: usize = 1023;
/// maximum number of bit-coded registers, 1 bit per interrupt
const MAX_BIT_REGS: usize = MAX_INTERRUPTS.div_ceil(32);

/// PLIC registers for *machine mode* context only at this time.
///
/// The spec defines extra sets of registers for additional contexts,
/// that is supervisor, user and other modes, but these aren't supported
/// by the current code.
#[repr(C)]
pub struct PlicRegisters {
    /// Interrupt Priority Register
    _reserved0: u32,
    priority: [ReadWrite<u32, priority::Register>; MAX_INTERRUPTS],
    _reserved1: [u8; 0x1000 - (MAX_INTERRUPTS + 1) * 4],
    /// Interrupt Pending Register
    pending: [ReadOnly<u32>; MAX_BIT_REGS],
    _reserved2: [u8; 0x1000 - MAX_BIT_REGS * 4],
    /// Interrupt Enable Register
    enable: [ReadWrite<u32>; MAX_BIT_REGS],
    _reserved3: [u8; 0x20_0000 - 0x2000 - MAX_BIT_REGS * 4],
    /// Priority Threshold Register
    threshold: ReadWrite<u32, priority::Register>,
    /// Claim/Complete Register
    claim: ReadWrite<u32>,
}

/// Check that the registers are aligned to the PLIC memory map
const _: () = assert!(core::mem::offset_of!(PlicRegisters, priority) == 0x4);
const _: () = assert!(core::mem::offset_of!(PlicRegisters, pending) == 0x1000);
const _: () = assert!(core::mem::offset_of!(PlicRegisters, enable) == 0x2000);
const _: () = assert!(core::mem::offset_of!(PlicRegisters, threshold) == 0x20_0000);
const _: () = assert!(core::mem::offset_of!(PlicRegisters, claim) == 0x20_0004);

/// A wrapper around the PLIC registers to provide safe access to the registers
/// within the defined interrupt number range
struct RegsWrapper {
    registers: StaticRef<PlicRegisters>,
    total_ints: usize,
}

impl RegsWrapper {
    const fn new(registers: StaticRef<PlicRegisters>, total_ints: usize) -> Self {
        Self {
            registers,
            total_ints,
        }
    }

    fn get_enable_regs(&self) -> &[ReadWrite<u32>] {
        // One bit per interrupt, total number of registers is
        // the number of interrupts divided by 32 rounded up
        &self.registers.enable[0..self.total_ints.div_ceil(32)]
    }

    // Unused by the current code
    #[allow(dead_code)]
    fn get_pending_regs(&self) -> &[ReadOnly<u32>] {
        // One bit per interrupt, total number of registers is
        // the number of interrupts divided by 32 rounded up
        &self.registers.pending[0..self.total_ints.div_ceil(32)]
    }

    fn get_priority_regs(&self) -> &[ReadWrite<u32, priority::Register>] {
        // One 32-bit register per interrupt source
        &self.registers.priority[0..self.total_ints]
    }

    fn get_threshold_reg(&self) -> &ReadWrite<u32, priority::Register> {
        &self.registers.threshold
    }

    fn get_claim_reg(&self) -> &ReadWrite<u32> {
        &self.registers.claim
    }
}

register_bitfields![u32,
    priority [
        Priority OFFSET(0) NUMBITS(3) []
    ]
];

/// The PLIC instance generic parameter indicates the total number of
/// interrupt sources implemented on the specific chip.
///
/// 51 is a default for backwards compatibility with the SiFive based
/// platforms implemented without the generic parameter.
pub struct Plic<const TOTAL_INTS: usize = 51> {
    registers: RegsWrapper,
    saved: [VolatileCell<LocalRegisterCopy<u32>>; 2],
}

impl<const TOTAL_INTS: usize> Plic<TOTAL_INTS> {
    pub const fn new(base: StaticRef<PlicRegisters>) -> Self {
        Plic {
            registers: RegsWrapper::new(base, TOTAL_INTS),
            saved: [
                VolatileCell::new(LocalRegisterCopy::new(0)),
                VolatileCell::new(LocalRegisterCopy::new(0)),
            ],
        }
    }

    /// Clear all pending interrupts. The [`PLIC specification`] section 7:
    /// > A successful claim will also atomically clear the corresponding pending bit on the interrupt source..
    /// Note that this function will only clear the enabled interrupt sources, as only those can be claimed.
    /// [`PLIC specification`]: <https://github.com/riscv/riscv-plic-spec/blob/master/riscv-plic.adoc>
    pub fn clear_all_pending(&self) {
        let claim = self.registers.get_claim_reg();
        loop {
            let id = claim.get();
            if id == 0 {
                break;
            }
            claim.set(id);
        }
    }

    /// Enable a list of interrupt IDs. The IDs must be in the range 1..TOTAL_INTS.
    pub fn enable_specific_interrupts(&self, interrupts: &[u32]) {
        let enable_regs = self.registers.get_enable_regs();
        for interrupt in interrupts {
            let offset = interrupt / 32;
            let irq = interrupt % 32;
            let old_value = enable_regs[offset as usize].get();
            enable_regs[offset as usize].set(old_value | (1 << irq));

            // Set some default priority for each interrupt. This is not really used
            // at this point.
            // The priority registers indexed 0 for interrupt 1, 1 for interrupt 2, etc.
            // so we subtract 1 from the interrupt number to get the correct index.
            self.registers.get_priority_regs()[*interrupt as usize - 1]
                .write(priority::Priority.val(4));
        }
        // Accept all interrupts.
        self.registers
            .get_threshold_reg()
            .write(priority::Priority.val(0));
    }

    pub fn disable_specific_interrupts(&self, interrupts: &[u32]) {
        let enable_regs = self.registers.get_enable_regs();
        for interrupt in interrupts {
            let offset = interrupt / 32;
            let irq = interrupt % 32;
            let old_value = enable_regs[offset as usize].get();
            enable_regs[offset as usize].set(old_value & !(1 << irq));
        }
    }

    /// Enable all interrupts.
    pub fn enable_all(&self) {
        let enable_regs = self.registers.get_enable_regs();
        let priority_regs = &self.registers.get_priority_regs();

        for enable in enable_regs.iter() {
            enable.set(0xFFFF_FFFF);
        }

        // Set some default priority for each interrupt. This is not really used
        // at this point.
        for priority in priority_regs.iter() {
            priority.write(priority::Priority.val(4));
        }

        // Accept all interrupts.
        self.registers
            .get_threshold_reg()
            .write(priority::Priority.val(0));
    }

    /// Disable all interrupts.
    pub fn disable_all(&self) {
        let enable_regs = self.registers.get_enable_regs();

        for enable in enable_regs.iter() {
            enable.set(0);
        }
    }

    /// Get the index (0-256) of the lowest number pending interrupt, or `None` if
    /// none is pending. RISC-V PLIC has a "claim" register which makes it easy
    /// to grab the highest priority pending interrupt.
    pub fn next_pending(&self) -> Option<u32> {
        let claim = self.registers.get_claim_reg().get();
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
        self.registers.get_claim_reg().set(index);

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
        self.registers
            .get_threshold_reg()
            .write(priority::Priority.val(0));
    }
}
