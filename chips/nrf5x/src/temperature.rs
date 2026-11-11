// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Temperature sensor driver, nRF5X-family
//!
//! Generates a simple temperature measurement without sampling
//!
//! Authors
//! -------------------
//! * Niklas Adolfsson <niklasadolfsson1@gmail.com>
//! * Fredrik Nilsson <frednils@student.chalmers.se>
//! * Date: March 03, 2017

use kernel::ErrorCode;
use kernel::utilities::StaticRef;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{
    ReadOnly, ReadWrite, WriteOnly, register_bitfields, register_structs,
};

#[cfg(feature = "nrf52")]
register_structs! {
    pub TempRegisters {
        /// Start temperature measurement
        /// Address: 0x000 - 0x004
        (0x000 => pub task_start: WriteOnly<u32, Task::Register>),
        /// Stop temperature measurement
        /// Address: 0x004 - 0x008
        (0x004 => pub task_stop: WriteOnly<u32, Task::Register>),
        /// Reserved
        (0x008 => _reserved1: [u32; 62]),
        /// Temperature measurement complete, data ready
        /// Address: 0x100 - 0x104
        (0x100 => pub event_datardy: ReadWrite<u32, Event::Register>),
        /// Reserved
        (0x104 => _reserved2: [u32; 128]),
        /// Enable interrupt
        /// Address: 0x304 - 0x308
        (0x304 => pub intenset: ReadWrite<u32, Intenset::Register>),
        /// Disable interrupt
        /// Address: 0x308 - 0x30c
        (0x308 => pub intenclr: ReadWrite<u32, Intenclr::Register>),
        /// Reserved
        (0x30C => _reserved3: [u32; 127]),
        /// Temperature in °C (0.25° steps)
        /// Address: 0x508 - 0x50c
        (0x508 => pub temp: ReadOnly<u32, Temperature::Register>),
        /// Reserved
        (0x50C => _reserved4: [u32; 5]),
        /// Slope of piece wise linear function (nRF52 only)
        /// Address 0x520 - 0x534
        (0x520 => pub a: [ReadWrite<u32, A::Register>; 6]),
        (0x538 => _reserved5: [u32; 2]),
        /// y-intercept of 5th piece wise linear function (nRF52 only)
        /// Address: 0x540 - 0x554
        (0x540 => pub b: [ReadWrite<u32, B::Register>; 6]),
        (0x558 => _reserved6: [u32; 2]),
        /// End point of 1st piece wise linear function (nRF52 only)
        /// Address: 0x560 - 0x570
        (0x560 => pub t: [ReadWrite<u32, B::Register>; 5]),
        (0x574 => @END),
    }
}

#[cfg(not(feature = "nrf52"))]
register_structs! {
    pub TempRegisters {
        /// Start temperature measurement
        /// Address: 0x000 - 0x004
        (0x000 => pub task_start: WriteOnly<u32, Task::Register>),
        /// Stop temperature measurement
        /// Address: 0x004 - 0x008
        (0x004 => pub task_stop: WriteOnly<u32, Task::Register>),
        /// Reserved
        (0x008 => _reserved1: [u32; 62]),
        /// Temperature measurement complete, data ready
        /// Address: 0x100 - 0x104
        (0x100 => pub event_datardy: ReadWrite<u32, Event::Register>),
        /// Reserved
        (0x104 => _reserved2: [u32; 128]),
        /// Enable interrupt
        /// Address: 0x304 - 0x308
        (0x304 => pub intenset: ReadWrite<u32, Intenset::Register>),
        /// Disable interrupt
        /// Address: 0x308 - 0x30c
        (0x308 => pub intenclr: ReadWrite<u32, Intenclr::Register>),
        /// Reserved
        (0x30C => _reserved3: [u32; 127]),
        /// Temperature in °C (0.25° steps)
        /// Address: 0x508 - 0x50c
        (0x508 => pub temp: ReadOnly<u32, Temperature::Register>),
        /// Reserved
        (0x50C => _reserved4: [u32; 5]),
        (0x520 => _reserved5: [u32; 2]),
        (0x528 => _reserved6: [u32; 2]),
        (0x530 => @END),
    }
}

register_bitfields! [u32,
    /// Start task
    Task [
        ENABLE OFFSET(0) NUMBITS(1)
    ],

    /// Read event
    Event [
        READY OFFSET(0) NUMBITS(1)
    ],

    /// Enabled interrupt
    Intenset [
        DATARDY OFFSET(0) NUMBITS(1)
    ],

    /// Disable interrupt
    Intenclr [
        DATARDY OFFSET(0) NUMBITS(1)
    ],

    /// Temperature in °C (0.25° steps)
    Temperature [
        TEMP OFFSET(0) NUMBITS(32)
    ],

    /// Slope of piece wise linear function
    A [
        SLOPE OFFSET(0) NUMBITS(12)
    ],

    /// y-intercept of wise linear function
    B [
        INTERCEPT OFFSET(0) NUMBITS(14)
    ],

    /// End point of wise linear function
    T [
       PIECE OFFSET(0) NUMBITS(8)
    ]
];

pub struct Temp<'a> {
    registers: StaticRef<TempRegisters>,
    client: OptionalCell<&'a dyn kernel::hil::sensors::TemperatureClient>,
}

impl<'a> Temp<'a> {
    pub const fn new(registers: StaticRef<TempRegisters>) -> Temp<'a> {
        Temp {
            registers,
            client: OptionalCell::empty(),
        }
    }

    /// Temperature interrupt handler
    pub fn handle_interrupt(&self) {
        // disable interrupts
        self.disable_interrupts();

        // get temperature
        // Result of temperature measurement in °C, 2's complement format, 0.25 °C steps
        let temp = (self.registers.temp.get() as i32 * 100) / 4;

        // stop measurement
        self.registers.task_stop.write(Task::ENABLE::SET);

        // disable interrupts
        self.disable_interrupts();

        // trigger callback with temperature
        self.client.map(|client| client.callback(Ok(temp)));
    }

    fn enable_interrupts(&self) {
        self.registers.intenset.write(Intenset::DATARDY::SET);
    }

    fn disable_interrupts(&self) {
        self.registers.intenclr.write(Intenclr::DATARDY::SET);
    }
}

impl<'a> kernel::hil::sensors::TemperatureDriver<'a> for Temp<'a> {
    fn read_temperature(&self) -> Result<(), ErrorCode> {
        self.enable_interrupts();
        self.registers.event_datardy.write(Event::READY::CLEAR);
        self.registers.task_start.write(Task::ENABLE::SET);
        Ok(())
    }

    fn set_client(&self, client: &'a dyn kernel::hil::sensors::TemperatureClient) {
        self.client.set(client);
    }
}
