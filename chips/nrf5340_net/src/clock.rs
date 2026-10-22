// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Clock control for the nRF5340 network core.

use kernel::utilities::StaticRef;
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{ReadWrite, WriteOnly, register_bitfields, register_structs};

register_structs! {
    pub ClockRegisters {
        (0x000 => tasks_hfclkstart: WriteOnly<u32, Task::Register>),
        (0x004 => tasks_hfclkstop: WriteOnly<u32, Task::Register>),
        (0x008 => tasks_lfclkstart: WriteOnly<u32, Task::Register>),
        (0x00C => tasks_lfclkstop: WriteOnly<u32, Task::Register>),
        (0x010 => _reserved0),
        (0x100 => events_hfclkstarted: ReadWrite<u32, Event::Register>),
        (0x104 => events_lfclkstarted: ReadWrite<u32, Event::Register>),
        (0x108 => @END),
    }
}

register_bitfields! [u32,
    Task [
        ENABLE OFFSET(0) NUMBITS(1)
    ],
    Event [
        READY OFFSET(0) NUMBITS(1)
    ]
];

pub struct Clock {
    registers: StaticRef<ClockRegisters>,
}

impl Clock {
    pub const fn new(registers: StaticRef<ClockRegisters>) -> Self {
        Self { registers }
    }

    /// Start the HFXO and the LFCLK, busy-waiting until both are running.
    ///
    /// The LFCLK runs from its reset source, the internal RC oscillator.
    pub fn start(&self) {
        self.registers.tasks_lfclkstop.write(Task::ENABLE::SET);
        self.registers.tasks_hfclkstop.write(Task::ENABLE::SET);
        self.registers
            .events_lfclkstarted
            .write(Event::READY::CLEAR);
        self.registers
            .events_hfclkstarted
            .write(Event::READY::CLEAR);
        self.registers.tasks_lfclkstart.write(Task::ENABLE::SET);
        self.registers.tasks_hfclkstart.write(Task::ENABLE::SET);
        while !self.registers.events_lfclkstarted.is_set(Event::READY) {}
        while !self.registers.events_hfclkstarted.is_set(Event::READY) {}
    }
}
