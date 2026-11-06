// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Interprocessor communication (IPC), nRF53
//!
//! The IPC peripherals of the two nRF5340 cores signal each other over 16
//! channels. This driver sends on channel `n` with task `n`, and receives
//! channel `n` with event `n`. Data travels through RAM both cores can access;
//! `send` and the receive callback order those accesses against the signal.

use core::sync::atomic::{Ordering, fence};
use kernel::utilities::StaticRef;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{ReadWrite, WriteOnly, register_bitfields, register_structs};

pub const NUM_CHANNELS: usize = 16;

register_structs! {
    pub IpcRegisters {
        (0x000 => tasks_send: [WriteOnly<u32, Task::Register>; NUM_CHANNELS]),
        (0x040 => _reserved0),
        (0x100 => events_receive: [ReadWrite<u32, Event::Register>; NUM_CHANNELS]),
        (0x140 => _reserved1),
        (0x304 => intenset: ReadWrite<u32>),
        (0x308 => _reserved2),
        (0x510 => send_cnf: [ReadWrite<u32>; NUM_CHANNELS]),
        (0x550 => _reserved3),
        (0x590 => receive_cnf: [ReadWrite<u32>; NUM_CHANNELS]),
        (0x5D0 => @END),
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

pub trait IpcClient {
    /// The other core signalled `channel`.
    fn received(&self, channel: usize);
}

pub struct Ipc<'a> {
    registers: StaticRef<IpcRegisters>,
    client: OptionalCell<&'a dyn IpcClient>,
}

impl<'a> Ipc<'a> {
    pub const fn new(registers: StaticRef<IpcRegisters>) -> Self {
        Self {
            registers,
            client: OptionalCell::empty(),
        }
    }

    pub fn set_client(&self, client: &'a dyn IpcClient) {
        self.client.set(client);
    }

    /// Interrupt when the other core signals `channel`, which must be below
    /// `NUM_CHANNELS`.
    pub fn listen(&self, channel: usize) {
        self.registers.receive_cnf[channel].set(1 << channel);
        self.registers.intenset.set(1 << channel);
    }

    /// Signal the other core on `channel`, which must be below `NUM_CHANNELS`.
    pub fn send(&self, channel: usize) {
        // Make shared-RAM writes visible to the other core before it is told.
        fence(Ordering::SeqCst);
        self.registers.send_cnf[channel].set(1 << channel);
        self.registers.tasks_send[channel].write(Task::ENABLE::SET);
    }

    pub fn handle_interrupt(&self) {
        for channel in 0..NUM_CHANNELS {
            if self.registers.events_receive[channel].is_set(Event::READY) {
                self.registers.events_receive[channel].write(Event::READY::CLEAR);
                // Read shared RAM only after seeing the signal.
                fence(Ordering::SeqCst);
                self.client.map(|client| client.received(channel));
            }
        }
    }
}
