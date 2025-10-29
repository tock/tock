// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! TRNG driver, nRF5X-family
//!
//! The TRNG generates 1 byte randomness at the time value in the interval
//! 0 <= r <= 255.
//!
//! Because that the he capsule requires 4 bytes of randomness at the time.
//! 4 bytes of randomness must be generated before returning  back to capsule.
//!
//! Therefore this module will have to use the TRNG four times.
//! A counter `index` has been introduced to keep track of this.
//! The four bytes of randomness is stored in a `Cell<u32>` which shifted
//! according to append one byte at the time.
//!
//! In the current implementation if done > 4 for some strange reason the
//! random generation will be restarted
//!
//! Authors
//! -------------------
//! * Niklas Adolfsson <niklasadolfsson1@gmail.com>
//! * Fredrik Nilsson <frednils@student.chalmers.se>
//! * Date: March 01, 2017

use core::cell::Cell;
use kernel::hil::entropy::{self, Continue};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, ReadOnly, ReadWrite, WriteOnly};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

#[repr(C)]
pub struct RngRegisters {
    /// Task starting the random number generator
    /// Address: 0x000 - 0x004
    pub task_start: WriteOnly<u32, Task::Register>,
    /// Task stopping the random number generator
    /// Address: 0x004 - 0x008
    pub task_stop: WriteOnly<u32, Task::Register>,
    /// Reserved
    _reserved1: [u32; 62],
    /// Event being generated for every new random number written to the VALUE register
    /// Address: 0x100 - 0x104
    pub event_valrdy: ReadWrite<u32, Event::Register>,
    /// Reserved
    _reserved2: [u32; 63],
    /// Shortcut register
    /// Address: 0x200 - 0x204
    pub shorts: ReadWrite<u32, Shorts::Register>,
    _reserved3: [u32; 64],
    /// Enable interrupt
    /// Address: 0x304 - 0x308
    pub intenset: ReadWrite<u32, Intenset::Register>,
    /// Disable interrupt
    /// Address: 0x308 - 0x30c
    pub intenclr: ReadWrite<u32, Intenclr::Register>,
    _reserved4: [u32; 126],
    /// Configuration register
    /// Address: 0x504 - 0x508
    pub config: ReadWrite<u32, Config::Register>,
    /// Output random number
    /// Address: 0x508 - 0x50c
    pub value: ReadOnly<u32, Value::Register>,
}

register_bitfields! [u32,
    /// Start task
    Task [
        ENABLE OFFSET(0) NUMBITS(1)
    ],

    /// Ready event
    Event [
        READY OFFSET(0) NUMBITS(1)
    ],

    /// Shortcut register
    Shorts [
        /// Shortcut between VALRDY event and STOP task
        VALRDY_STOP OFFSET(0) NUMBITS(1)
    ],

    /// Enable interrupt
    Intenset [
        VALRDY OFFSET(0) NUMBITS(1)
    ],

    /// Disable interrupt
    Intenclr [
        VALRDY OFFSET(0) NUMBITS(1)
    ],

    /// Configuration register
    Config [
        /// Bias correction
        DERCEN OFFSET(0) NUMBITS(32)
    ],

    /// Output random number
    Value [
        /// Generated random number
        VALUE OFFSET(0) NUMBITS(8)
    ]
];

pub struct Trng<'a> {
    registers: StaticRef<RngRegisters>,
    client: OptionalCell<&'a dyn entropy::Client32>,
    index: Cell<usize>,
    randomness: Cell<u32>,
}

impl<'a> Trng<'a> {
    pub const fn new(registers: StaticRef<RngRegisters>) -> Trng<'a> {
        Trng {
            registers,
            client: OptionalCell::empty(),
            index: Cell::new(0),
            randomness: Cell::new(0),
        }
    }

    /// RNG Interrupt handler
    pub fn handle_interrupt(&self) {
        self.disable_interrupts();

        match self.index.get() {
            // fetch more data need 4 bytes because the capsule requires that
            e @ 0..=3 => {
                // 3 lines below to change data in Cell, perhaps it can be done more nicely
                let mut rn = self.randomness.get();
                // 1 byte randomness
                let r = self.registers.value.get();
                //  e = 0 -> byte 1 LSB
                //  e = 1 -> byte 2
                //  e = 2 -> byte 3
                //  e = 3 -> byte 4 MSB
                rn |= r << (8 * e);
                self.randomness.set(rn);

                self.index.set(e + 1);
                self.start_rng()
            }
            // fetched 4 bytes of data generated, then notify the capsule
            4 => {
                self.client.map(|client| {
                    let result = client.entropy_available(&mut TrngIter(self), Ok(()));
                    if Continue::Done != result {
                        // need more randomness i.e generate more randomness
                        self.start_rng();
                    }
                });
            }
            // This should never happen if the logic is correct
            // Restart randomness generation if this condition occurs
            _ => {
                self.index.set(0);
                self.randomness.set(0);
            }
        }
    }

    fn enable_interrupts(&self) {
        self.registers.intenset.write(Intenset::VALRDY::SET);
    }

    fn disable_interrupts(&self) {
        self.registers.intenclr.write(Intenclr::VALRDY::SET);
    }

    fn start_rng(&self) {
        // Reset `valrdy`
        self.registers.event_valrdy.write(Event::READY::CLEAR);

        // Enable interrupts
        self.enable_interrupts();

        // Start rng
        self.registers.task_start.write(Task::ENABLE::SET);
    }
}

struct TrngIter<'a, 'b: 'a>(&'a Trng<'b>);

impl Iterator for TrngIter<'_, '_> {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        if self.0.index.get() == 4 {
            let rn = self.0.randomness.get();
            // indicate 4 bytes of randomness taken by the capsule
            self.0.index.set(0);
            self.0.randomness.set(0);
            Some(rn)
        } else {
            None
        }
    }
}

impl<'a> entropy::Entropy32<'a> for Trng<'a> {
    fn get(&self) -> Result<(), ErrorCode> {
        self.start_rng();
        Ok(())
    }

    fn cancel(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn set_client(&'a self, client: &'a dyn entropy::Client32) {
        self.client.set(client);
    }
}
