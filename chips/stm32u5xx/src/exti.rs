// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2026.

use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::utilities::StaticRef;

register_structs! {
    pub ExtiRegisters {
        /// Rising trigger selection register
        (0x000 => pub rtsr1: ReadWrite<u32, LineReg::Register>),
        /// Falling trigger selection register
        (0x004 => pub ftsr1: ReadWrite<u32, LineReg::Register>),
        /// Software interrupt event register
        (0x008 => pub swier1: ReadWrite<u32, LineReg::Register>),
        /// Pending register 1 (Rising)
        (0x00C => pub rpr1: ReadWrite<u32, LineReg::Register>),
        /// Pending register 1 (Falling)
        (0x010 => pub fpr1: ReadWrite<u32, LineReg::Register>),
        /// Security configuration register
        (0x014 => pub seccfgr1: ReadWrite<u32, LineReg::Register>),
        (0x018 => _reserved0: [u32; 18]),
        /// External interrupt selection registers
        (0x060 => pub exticr: [ReadWrite<u32, EXTICR::Register>; 4]),
        (0x070 => _reserved1: [u32; 4]),
        /// Interrupt mask register
        (0x080 => pub imr1: ReadWrite<u32, LineReg::Register>),
        (0x084 => @END),
    }
}

register_bitfields![u32,
    pub LineReg [
        L0 OFFSET(0) NUMBITS(1) [],
        L1 OFFSET(1) NUMBITS(1) [],
        L2 OFFSET(2) NUMBITS(1) [],
        L3 OFFSET(3) NUMBITS(1) [],
        L4 OFFSET(4) NUMBITS(1) [],
        L5 OFFSET(5) NUMBITS(1) [],
        L6 OFFSET(6) NUMBITS(1) [],
        L7 OFFSET(7) NUMBITS(1) [],
        L8 OFFSET(8) NUMBITS(1) [],
        L9 OFFSET(9) NUMBITS(1) [],
        L10 OFFSET(10) NUMBITS(1) [],
        L11 OFFSET(11) NUMBITS(1) [],
        L12 OFFSET(12) NUMBITS(1) [],
        L13 OFFSET(13) NUMBITS(1) [],
        L14 OFFSET(14) NUMBITS(1) [],
        L15 OFFSET(15) NUMBITS(1) []
    ],
    pub EXTICR [
        EXTI0 OFFSET(0) NUMBITS(8) [],
        EXTI1 OFFSET(8) NUMBITS(8) [],
        EXTI2 OFFSET(16) NUMBITS(8) [],
        EXTI3 OFFSET(24) NUMBITS(8) []
    ]
];

/// Base address for EXTI in Secure Alias mode.
pub const EXTI_BASE: StaticRef<ExtiRegisters> =
    unsafe { StaticRef::new(0x56022000 as *const ExtiRegisters) };

#[derive(Copy, Clone, PartialEq)]
/// Identifiers for the 16 external interrupt lines (EXTI0 - EXTI15).
pub enum LineId {
    Line00 = 0,
    Line01 = 1,
    Line02 = 2,
    Line03 = 3,
    Line04 = 4,
    Line05 = 5,
    Line06 = 6,
    Line07 = 7,
    Line08 = 8,
    Line09 = 9,
    Line10 = 10,
    Line11 = 11,
    Line12 = 12,
    Line13 = 13,
    Line14 = 14,
    Line15 = 15,
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
        self.clear_pending(line);

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
        let field_index = line_num % 4;

        let reg = &self.registers.exticr[register_index];
        match field_index {
            0 => reg.modify(EXTICR::EXTI0.val(port)),
            1 => reg.modify(EXTICR::EXTI1.val(port)),
            2 => reg.modify(EXTICR::EXTI2.val(port)),
            3 => reg.modify(EXTICR::EXTI3.val(port)),
            _ => unreachable!(),
        }
    }

    pub(crate) fn set_secure(&self, line: LineId) {
        match line {
            LineId::Line00 => self.registers.seccfgr1.modify(LineReg::L0::SET),
            LineId::Line01 => self.registers.seccfgr1.modify(LineReg::L1::SET),
            LineId::Line02 => self.registers.seccfgr1.modify(LineReg::L2::SET),
            LineId::Line03 => self.registers.seccfgr1.modify(LineReg::L3::SET),
            LineId::Line04 => self.registers.seccfgr1.modify(LineReg::L4::SET),
            LineId::Line05 => self.registers.seccfgr1.modify(LineReg::L5::SET),
            LineId::Line06 => self.registers.seccfgr1.modify(LineReg::L6::SET),
            LineId::Line07 => self.registers.seccfgr1.modify(LineReg::L7::SET),
            LineId::Line08 => self.registers.seccfgr1.modify(LineReg::L8::SET),
            LineId::Line09 => self.registers.seccfgr1.modify(LineReg::L9::SET),
            LineId::Line10 => self.registers.seccfgr1.modify(LineReg::L10::SET),
            LineId::Line11 => self.registers.seccfgr1.modify(LineReg::L11::SET),
            LineId::Line12 => self.registers.seccfgr1.modify(LineReg::L12::SET),
            LineId::Line13 => self.registers.seccfgr1.modify(LineReg::L13::SET),
            LineId::Line14 => self.registers.seccfgr1.modify(LineReg::L14::SET),
            LineId::Line15 => self.registers.seccfgr1.modify(LineReg::L15::SET),
        }
    }

    pub(crate) fn mask_interrupt(&self, line: LineId) {
        match line {
            LineId::Line00 => self.registers.imr1.modify(LineReg::L0::CLEAR),
            LineId::Line01 => self.registers.imr1.modify(LineReg::L1::CLEAR),
            LineId::Line02 => self.registers.imr1.modify(LineReg::L2::CLEAR),
            LineId::Line03 => self.registers.imr1.modify(LineReg::L3::CLEAR),
            LineId::Line04 => self.registers.imr1.modify(LineReg::L4::CLEAR),
            LineId::Line05 => self.registers.imr1.modify(LineReg::L5::CLEAR),
            LineId::Line06 => self.registers.imr1.modify(LineReg::L6::CLEAR),
            LineId::Line07 => self.registers.imr1.modify(LineReg::L7::CLEAR),
            LineId::Line08 => self.registers.imr1.modify(LineReg::L8::CLEAR),
            LineId::Line09 => self.registers.imr1.modify(LineReg::L9::CLEAR),
            LineId::Line10 => self.registers.imr1.modify(LineReg::L10::CLEAR),
            LineId::Line11 => self.registers.imr1.modify(LineReg::L11::CLEAR),
            LineId::Line12 => self.registers.imr1.modify(LineReg::L12::CLEAR),
            LineId::Line13 => self.registers.imr1.modify(LineReg::L13::CLEAR),
            LineId::Line14 => self.registers.imr1.modify(LineReg::L14::CLEAR),
            LineId::Line15 => self.registers.imr1.modify(LineReg::L15::CLEAR),
        }
    }

    pub(crate) fn unmask_interrupt(&self, line: LineId) {
        match line {
            LineId::Line00 => self.registers.imr1.modify(LineReg::L0::SET),
            LineId::Line01 => self.registers.imr1.modify(LineReg::L1::SET),
            LineId::Line02 => self.registers.imr1.modify(LineReg::L2::SET),
            LineId::Line03 => self.registers.imr1.modify(LineReg::L3::SET),
            LineId::Line04 => self.registers.imr1.modify(LineReg::L4::SET),
            LineId::Line05 => self.registers.imr1.modify(LineReg::L5::SET),
            LineId::Line06 => self.registers.imr1.modify(LineReg::L6::SET),
            LineId::Line07 => self.registers.imr1.modify(LineReg::L7::SET),
            LineId::Line08 => self.registers.imr1.modify(LineReg::L8::SET),
            LineId::Line09 => self.registers.imr1.modify(LineReg::L9::SET),
            LineId::Line10 => self.registers.imr1.modify(LineReg::L10::SET),
            LineId::Line11 => self.registers.imr1.modify(LineReg::L11::SET),
            LineId::Line12 => self.registers.imr1.modify(LineReg::L12::SET),
            LineId::Line13 => self.registers.imr1.modify(LineReg::L13::SET),
            LineId::Line14 => self.registers.imr1.modify(LineReg::L14::SET),
            LineId::Line15 => self.registers.imr1.modify(LineReg::L15::SET),
        }
    }

    pub(crate) fn clear_pending(&self, line: LineId) {
        match line {
            LineId::Line00 => {
                self.registers.rpr1.write(LineReg::L0::SET);
                self.registers.fpr1.write(LineReg::L0::SET);
            }
            LineId::Line01 => {
                self.registers.rpr1.write(LineReg::L1::SET);
                self.registers.fpr1.write(LineReg::L1::SET);
            }
            LineId::Line02 => {
                self.registers.rpr1.write(LineReg::L2::SET);
                self.registers.fpr1.write(LineReg::L2::SET);
            }
            LineId::Line03 => {
                self.registers.rpr1.write(LineReg::L3::SET);
                self.registers.fpr1.write(LineReg::L3::SET);
            }
            LineId::Line04 => {
                self.registers.rpr1.write(LineReg::L4::SET);
                self.registers.fpr1.write(LineReg::L4::SET);
            }
            LineId::Line05 => {
                self.registers.rpr1.write(LineReg::L5::SET);
                self.registers.fpr1.write(LineReg::L5::SET);
            }
            LineId::Line06 => {
                self.registers.rpr1.write(LineReg::L6::SET);
                self.registers.fpr1.write(LineReg::L6::SET);
            }
            LineId::Line07 => {
                self.registers.rpr1.write(LineReg::L7::SET);
                self.registers.fpr1.write(LineReg::L7::SET);
            }
            LineId::Line08 => {
                self.registers.rpr1.write(LineReg::L8::SET);
                self.registers.fpr1.write(LineReg::L8::SET);
            }
            LineId::Line09 => {
                self.registers.rpr1.write(LineReg::L9::SET);
                self.registers.fpr1.write(LineReg::L9::SET);
            }
            LineId::Line10 => {
                self.registers.rpr1.write(LineReg::L10::SET);
                self.registers.fpr1.write(LineReg::L10::SET);
            }
            LineId::Line11 => {
                self.registers.rpr1.write(LineReg::L11::SET);
                self.registers.fpr1.write(LineReg::L11::SET);
            }
            LineId::Line12 => {
                self.registers.rpr1.write(LineReg::L12::SET);
                self.registers.fpr1.write(LineReg::L12::SET);
            }
            LineId::Line13 => {
                self.registers.rpr1.write(LineReg::L13::SET);
                self.registers.fpr1.write(LineReg::L13::SET);
            }
            LineId::Line14 => {
                self.registers.rpr1.write(LineReg::L14::SET);
                self.registers.fpr1.write(LineReg::L14::SET);
            }
            LineId::Line15 => {
                self.registers.rpr1.write(LineReg::L15::SET);
                self.registers.fpr1.write(LineReg::L15::SET);
            }
        }
    }

    pub(crate) fn select_rising_trigger(&self, line: LineId) {
        match line {
            LineId::Line00 => self.registers.rtsr1.modify(LineReg::L0::SET),
            LineId::Line01 => self.registers.rtsr1.modify(LineReg::L1::SET),
            LineId::Line02 => self.registers.rtsr1.modify(LineReg::L2::SET),
            LineId::Line03 => self.registers.rtsr1.modify(LineReg::L3::SET),
            LineId::Line04 => self.registers.rtsr1.modify(LineReg::L4::SET),
            LineId::Line05 => self.registers.rtsr1.modify(LineReg::L5::SET),
            LineId::Line06 => self.registers.rtsr1.modify(LineReg::L6::SET),
            LineId::Line07 => self.registers.rtsr1.modify(LineReg::L7::SET),
            LineId::Line08 => self.registers.rtsr1.modify(LineReg::L8::SET),
            LineId::Line09 => self.registers.rtsr1.modify(LineReg::L9::SET),
            LineId::Line10 => self.registers.rtsr1.modify(LineReg::L10::SET),
            LineId::Line11 => self.registers.rtsr1.modify(LineReg::L11::SET),
            LineId::Line12 => self.registers.rtsr1.modify(LineReg::L12::SET),
            LineId::Line13 => self.registers.rtsr1.modify(LineReg::L13::SET),
            LineId::Line14 => self.registers.rtsr1.modify(LineReg::L14::SET),
            LineId::Line15 => self.registers.rtsr1.modify(LineReg::L15::SET),
        }
    }

    pub(crate) fn deselect_rising_trigger(&self, line: LineId) {
        match line {
            LineId::Line00 => self.registers.rtsr1.modify(LineReg::L0::CLEAR),
            LineId::Line01 => self.registers.rtsr1.modify(LineReg::L1::CLEAR),
            LineId::Line02 => self.registers.rtsr1.modify(LineReg::L2::CLEAR),
            LineId::Line03 => self.registers.rtsr1.modify(LineReg::L3::CLEAR),
            LineId::Line04 => self.registers.rtsr1.modify(LineReg::L4::CLEAR),
            LineId::Line05 => self.registers.rtsr1.modify(LineReg::L5::CLEAR),
            LineId::Line06 => self.registers.rtsr1.modify(LineReg::L6::CLEAR),
            LineId::Line07 => self.registers.rtsr1.modify(LineReg::L7::CLEAR),
            LineId::Line08 => self.registers.rtsr1.modify(LineReg::L8::CLEAR),
            LineId::Line09 => self.registers.rtsr1.modify(LineReg::L9::CLEAR),
            LineId::Line10 => self.registers.rtsr1.modify(LineReg::L10::CLEAR),
            LineId::Line11 => self.registers.rtsr1.modify(LineReg::L11::CLEAR),
            LineId::Line12 => self.registers.rtsr1.modify(LineReg::L12::CLEAR),
            LineId::Line13 => self.registers.rtsr1.modify(LineReg::L13::CLEAR),
            LineId::Line14 => self.registers.rtsr1.modify(LineReg::L14::CLEAR),
            LineId::Line15 => self.registers.rtsr1.modify(LineReg::L15::CLEAR),
        }
    }

    pub(crate) fn select_falling_trigger(&self, line: LineId) {
        match line {
            LineId::Line00 => self.registers.ftsr1.modify(LineReg::L0::SET),
            LineId::Line01 => self.registers.ftsr1.modify(LineReg::L1::SET),
            LineId::Line02 => self.registers.ftsr1.modify(LineReg::L2::SET),
            LineId::Line03 => self.registers.ftsr1.modify(LineReg::L3::SET),
            LineId::Line04 => self.registers.ftsr1.modify(LineReg::L4::SET),
            LineId::Line05 => self.registers.ftsr1.modify(LineReg::L5::SET),
            LineId::Line06 => self.registers.ftsr1.modify(LineReg::L6::SET),
            LineId::Line07 => self.registers.ftsr1.modify(LineReg::L7::SET),
            LineId::Line08 => self.registers.ftsr1.modify(LineReg::L8::SET),
            LineId::Line09 => self.registers.ftsr1.modify(LineReg::L9::SET),
            LineId::Line10 => self.registers.ftsr1.modify(LineReg::L10::SET),
            LineId::Line11 => self.registers.ftsr1.modify(LineReg::L11::SET),
            LineId::Line12 => self.registers.ftsr1.modify(LineReg::L12::SET),
            LineId::Line13 => self.registers.ftsr1.modify(LineReg::L13::SET),
            LineId::Line14 => self.registers.ftsr1.modify(LineReg::L14::SET),
            LineId::Line15 => self.registers.ftsr1.modify(LineReg::L15::SET),
        }
    }

    pub(crate) fn deselect_falling_trigger(&self, line: LineId) {
        match line {
            LineId::Line00 => self.registers.ftsr1.modify(LineReg::L0::CLEAR),
            LineId::Line01 => self.registers.ftsr1.modify(LineReg::L1::CLEAR),
            LineId::Line02 => self.registers.ftsr1.modify(LineReg::L2::CLEAR),
            LineId::Line03 => self.registers.ftsr1.modify(LineReg::L3::CLEAR),
            LineId::Line04 => self.registers.ftsr1.modify(LineReg::L4::CLEAR),
            LineId::Line05 => self.registers.ftsr1.modify(LineReg::L5::CLEAR),
            LineId::Line06 => self.registers.ftsr1.modify(LineReg::L6::CLEAR),
            LineId::Line07 => self.registers.ftsr1.modify(LineReg::L7::CLEAR),
            LineId::Line08 => self.registers.ftsr1.modify(LineReg::L8::CLEAR),
            LineId::Line09 => self.registers.ftsr1.modify(LineReg::L9::CLEAR),
            LineId::Line10 => self.registers.ftsr1.modify(LineReg::L10::CLEAR),
            LineId::Line11 => self.registers.ftsr1.modify(LineReg::L11::CLEAR),
            LineId::Line12 => self.registers.ftsr1.modify(LineReg::L12::CLEAR),
            LineId::Line13 => self.registers.ftsr1.modify(LineReg::L13::CLEAR),
            LineId::Line14 => self.registers.ftsr1.modify(LineReg::L14::CLEAR),
            LineId::Line15 => self.registers.ftsr1.modify(LineReg::L15::CLEAR),
        }
    }

    pub(crate) fn is_pending(&self, line: LineId) -> bool {
        match line {
            LineId::Line00 => {
                self.registers.rpr1.is_set(LineReg::L0) || self.registers.fpr1.is_set(LineReg::L0)
            }
            LineId::Line01 => {
                self.registers.rpr1.is_set(LineReg::L1) || self.registers.fpr1.is_set(LineReg::L1)
            }
            LineId::Line02 => {
                self.registers.rpr1.is_set(LineReg::L2) || self.registers.fpr1.is_set(LineReg::L2)
            }
            LineId::Line03 => {
                self.registers.rpr1.is_set(LineReg::L3) || self.registers.fpr1.is_set(LineReg::L3)
            }
            LineId::Line04 => {
                self.registers.rpr1.is_set(LineReg::L4) || self.registers.fpr1.is_set(LineReg::L4)
            }
            LineId::Line05 => {
                self.registers.rpr1.is_set(LineReg::L5) || self.registers.fpr1.is_set(LineReg::L5)
            }
            LineId::Line06 => {
                self.registers.rpr1.is_set(LineReg::L6) || self.registers.fpr1.is_set(LineReg::L6)
            }
            LineId::Line07 => {
                self.registers.rpr1.is_set(LineReg::L7) || self.registers.fpr1.is_set(LineReg::L7)
            }
            LineId::Line08 => {
                self.registers.rpr1.is_set(LineReg::L8) || self.registers.fpr1.is_set(LineReg::L8)
            }
            LineId::Line09 => {
                self.registers.rpr1.is_set(LineReg::L9) || self.registers.fpr1.is_set(LineReg::L9)
            }
            LineId::Line10 => {
                self.registers.rpr1.is_set(LineReg::L10) || self.registers.fpr1.is_set(LineReg::L10)
            }
            LineId::Line11 => {
                self.registers.rpr1.is_set(LineReg::L11) || self.registers.fpr1.is_set(LineReg::L11)
            }
            LineId::Line12 => {
                self.registers.rpr1.is_set(LineReg::L12) || self.registers.fpr1.is_set(LineReg::L12)
            }
            LineId::Line13 => {
                self.registers.rpr1.is_set(LineReg::L13) || self.registers.fpr1.is_set(LineReg::L13)
            }
            LineId::Line14 => {
                self.registers.rpr1.is_set(LineReg::L14) || self.registers.fpr1.is_set(LineReg::L14)
            }
            LineId::Line15 => {
                self.registers.rpr1.is_set(LineReg::L15) || self.registers.fpr1.is_set(LineReg::L15)
            }
        }
    }
}
