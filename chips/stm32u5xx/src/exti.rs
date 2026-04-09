// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2026.

use enum_primitive::cast::FromPrimitive;
use enum_primitive::enum_from_primitive;
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{register_structs, ReadWrite};
use kernel::utilities::StaticRef;

register_structs! {
    pub ExtiRegisters {
        /// Rising trigger selection register
        (0x000 => pub rtsr1: ReadWrite<u32>),
        /// Falling trigger selection register
        (0x004 => pub ftsr1: ReadWrite<u32>),
        /// Software interrupt event register
        (0x008 => pub swier1: ReadWrite<u32>),
        /// Pending register 1 (Rising)
        (0x00C => pub rpr1: ReadWrite<u32>),
        /// Pending register 1 (Falling)
        (0x010 => pub fpr1: ReadWrite<u32>),
        /// Security configuration register
        (0x014 => pub seccfgr1: ReadWrite<u32>),
        (0x018 => _reserved0: [u32; 18]),
        /// External interrupt selection registers
        (0x060 => pub exticr: [ReadWrite<u32>; 4]),
        (0x070 => _reserved1: [u32; 4]),
        /// Interrupt mask register
        (0x080 => pub imr1: ReadWrite<u32>),
        (0x084 => @END),
    }
}

/// Base address for EXTI in Secure Alias mode.
pub const EXTI_BASE: StaticRef<ExtiRegisters> =
    unsafe { StaticRef::new(0x56022000 as *const ExtiRegisters) };

enum_from_primitive! {
    #[derive(Copy, Clone, PartialEq)]
    /// Identifiers for the 16 external interrupt lines (EXTI0 - EXTI15).
    pub enum LineId {
        Line00 = 0, Line01 = 1, Line02 = 2, Line03 = 3,
        Line04 = 4, Line05 = 5, Line06 = 6, Line07 = 7,
        Line08 = 8, Line09 = 9, Line10 = 10, Line11 = 11,
        Line12 = 12, Line13 = 13, Line14 = 14, Line15 = 15,
    }
}

/// The EXTI controller manages external interrupt lines and routes them
/// to registered clients (usually GPIO Pins).
pub struct Exti<'a> {
    registers: StaticRef<ExtiRegisters>,
    clients: [kernel::utilities::cells::OptionalCell<&'a dyn kernel::hil::gpio::Client>; 16],
}

impl<'a> Exti<'a> {
    /// Creates a new EXTI driver instance.
    pub const fn new(base: StaticRef<ExtiRegisters>) -> Self {
        Self {
            registers: base,
            clients: [const { kernel::utilities::cells::OptionalCell::empty() }; 16],
        }
    }

    /// Processes external interrupts and notifies registered clients.
    ///
    /// This is called from the chip's main ISR dispatcher. It clears the
    /// hardware pending flags and calls `fired()` on the associated client.
    pub fn handle_interrupt(&self, line: LineId) {
        let line_num = line as usize;
        
        // Clear pending flags
        self.registers.rpr1.set(1 << line_num);
        self.registers.fpr1.set(1 << line_num);

        // Notify the client
        self.clients[line_num].map(|client| {
            client.fired();
        });
    }

    pub(crate) fn register_client(&self, line: LineId, client: &'a dyn kernel::hil::gpio::Client) {
        let index = line as usize;
        if index < 16 {
            self.clients[index].set(client);
        }
    }

    pub(crate) fn select_port(&self, line: LineId, port: u32) {
        let line_num = line as usize;
        let register_index = line_num / 4;
        let offset = (line_num % 4) * 8;

        let mut val = self.registers.exticr[register_index].get();
        val &= !(0xFF << offset);
        val |= (port & 0xFF) << offset;
        self.registers.exticr[register_index].set(val);
    }

    pub(crate) fn set_secure(&self, line: LineId) {
        let val = self.registers.seccfgr1.get();
        self.registers.seccfgr1.set(val | (1 << (line as u32)));
    }

    pub(crate) fn mask_interrupt(&self, line: LineId) {
        let val = self.registers.imr1.get();
        self.registers.imr1.set(val & !(1 << (line as u32)));
    }

    pub(crate) fn unmask_interrupt(&self, line: LineId) {
        let val = self.registers.imr1.get();
        self.registers.imr1.set(val | (1 << (line as u32)));
    }

    pub(crate) fn clear_pending(&self, line: LineId) {
        self.registers.rpr1.set(1 << (line as u32));
        self.registers.fpr1.set(1 << (line as u32));
    }

    pub(crate) fn select_rising_trigger(&self, line: LineId) {
        let val = self.registers.rtsr1.get();
        self.registers.rtsr1.set(val | (1 << (line as u32)));
    }

    pub(crate) fn deselect_rising_trigger(&self, line: LineId) {
        let val = self.registers.rtsr1.get();
        self.registers.rtsr1.set(val & !(1 << (line as u32)));
    }

    pub(crate) fn select_falling_trigger(&self, line: LineId) {
        let val = self.registers.ftsr1.get();
        self.registers.ftsr1.set(val | (1 << (line as u32)));
    }

    pub(crate) fn deselect_falling_trigger(&self, line: LineId) {
        let val = self.registers.ftsr1.get();
        self.registers.ftsr1.set(val & !(1 << (line as u32)));
    }

    pub(crate) fn is_pending(&self, line: LineId) -> bool {
        (self.registers.rpr1.get() | self.registers.fpr1.get()) & (1 << (line as u32)) != 0
    }
}
