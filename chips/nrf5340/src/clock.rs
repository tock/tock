// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Clock control for the nRF5340 application core.

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
        (0x108 => _reserved1),
        (0x518 => lfclksrc: ReadWrite<u32, LfClkSrc::Register>),
        (0x51C => @END),
    }
}

register_bitfields! [u32,
    Task [
        ENABLE OFFSET(0) NUMBITS(1)
    ],
    Event [
        READY OFFSET(0) NUMBITS(1)
    ],
    LfClkSrc [
        SRC OFFSET(0) NUMBITS(2) [
            Lfrc = 1,
            Lfxo = 2
        ]
    ]
];

/// The source of the 32.768 kHz low-frequency clock (LFCLK).
pub enum LfclkSource {
    /// The internal RC oscillator.
    Lfrc,
    /// The crystal oscillator. Its crystal must be connected first, see
    /// `nrf5340::lfxo`.
    Lfxo,
}

pub struct Clock {
    registers: StaticRef<ClockRegisters>,
}

impl Clock {
    pub const fn new(registers: StaticRef<ClockRegisters>) -> Self {
        Self { registers }
    }

    /// Start the HFXO and the LFCLK from `lfclk`, busy-waiting until both are
    /// running.
    pub fn start(&self, lfclk: LfclkSource) {
        self.registers.tasks_lfclkstop.write(Task::ENABLE::SET);
        self.registers.tasks_hfclkstop.write(Task::ENABLE::SET);
        self.registers.lfclksrc.write(match lfclk {
            LfclkSource::Lfrc => LfClkSrc::SRC::Lfrc,
            LfclkSource::Lfxo => LfClkSrc::SRC::Lfxo,
        });
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
