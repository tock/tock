// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Support for legacy 8253-compatible timer.
//!
//! This module implements support for the 8253 programmable interrupt timer, aka the "PIT". This
//! device provides a source of periodic interrupts that can be used for timing purposes.
//!
//! The 8253 is rather old and has lots of quirks which are not found in more modern timer circuits.
//! As a result, some amount of jitter/drift is unavoidable when using the PIT. Sadly this is
//! unavoidable due to hardware limitation. For instance, the PIT's internal frequency is a rational
//! number but not an integer, so all calculations are imprecise due to integer rounding.
//!
//! This implementation is based on guidance from the following sources:
//!
//! * <https://wiki.osdev.org/Programmable_Interval_Timer>
//! * <https://en.wikipedia.org/wiki/Intel_8253>

use core::cell::Cell;

use kernel::ErrorCode;
use kernel::hil::time::{Alarm, AlarmClient, Frequency, Ticks, Ticks32, Time};
use tock_cells::numeric_cell_ext::NumericCellExt;
use tock_cells::optional_cell::OptionalCell;
use tock_registers_v0::{LocalRegisterCopy, Read, Write, register_bitfields, register_map};
use x86::registers::io::Port;

/// Frequency of the PIT's internal oscillator
///
/// According to Wikipedia, this should be "one third of the NTSC color subcarrier frequency."
///
/// Why such a specific value? This frequency is heavily used in analog television circuitry. At the
/// time the 8253 was first introduced, it was very easy and cheap to obtain a crystal oscillator at
/// exactly this frequency.
const OSCILLATOR_FREQUENCY: u32 = 3579545 / 3;

/// Frequency of the PIT timer
///
/// Parameter `R` is the PIT's reload value for channel 0.
///
/// Internal oscillator frequency is a rational, non-integer number. This means there is some loss
/// of precision because Tock uses integers to represent frequency.
pub struct PitFreq<const R: u16>;

impl<const R: u16> Frequency for PitFreq<R> {
    fn frequency() -> u32 {
        OSCILLATOR_FREQUENCY / (R as u32)
    }
}

/// Computes a PIT reload value for the given frequency.
///
/// The actual interrupt frequency will always be slightly different from the requested `freq` due
/// to hardware limitations. This function tries to get as close as possible.
pub const fn reload_value(freq: u32) -> u16 {
    // Lowest possible frequency is about 18 Hz
    if freq <= 18 {
        // PIT interprets zero as "maximum possible value"
        return 0x0000;
    }

    // Highest possible frequency is OSCILLATOR_FREQUENCY
    if freq >= OSCILLATOR_FREQUENCY {
        return 0x0001;
    }

    let mut r = OSCILLATOR_FREQUENCY / freq;

    // If remainder is more than half, then round up to get as close as possible
    if (OSCILLATOR_FREQUENCY % freq) >= (OSCILLATOR_FREQUENCY / 2) {
        r += 1;
    }

    r as u16
}

/// Reload value corresponding to 1 KHz
pub const RELOAD_1KHZ: u16 = reload_value(1000);

register_map! {
    #![bus(Port)]
    /// I/O ports for the PIT chip. This PIT starts at port 0x40.
    ports {
        0 => cd0: u8 { Read, Write },
        1 => cd1: u8 { Read, Write },
        2 => cd2: u8 { Read, Write },
        3 => mcr: u8 { Write },
    }
}

// Bitfield for the mode/command register
register_bitfields!(u8,
    PIT_MCR [
        BCD OFFSET(0) NUMBITS(1) [],
        MODE OFFSET(1) NUMBITS(3) [
            M2 = 0b010, // Rate generator
        ],
        ACCESS OFFSET(4) NUMBITS(2) [
            LOHI = 0b11, // Lobyte/hibyte access mode
        ],
        CHANNEL OFFSET(6) NUMBITS(2) [
            C0 = 0b00, // Channel 0
        ]
    ]
);

/// Timer based on 8253 "PIT" hardware
///
/// Although the PIT contains an internal counter, it is not a suitable source of ticks because it
/// runs at a high, fixed frequency and wraps very quickly. So instead, we configure the PIT to
/// generate periodic interrupts, and we increment an in-memory counter each time an interrupt
/// fires.
///
/// Parameter `R` is the reload value to use. This is loaded into a hardware register and determines
/// the interrupt frequency. Use [`reload_value`] to compute a reload value for the desired
/// frequency.
pub struct Pit<'a, const R: u16, P: ports::Interface = ports::Real> {
    alarm: OptionalCell<u32>,
    client: OptionalCell<&'a dyn AlarmClient>,
    now: Cell<usize>,
    ports: P,
}

impl<const R: u16> Pit<'_, R, ports::Real> {
    /// Creates a new PIT timer object.
    ///
    /// # Safety
    ///
    /// There must never be more than a single instance of `Pit` alive at any given time.
    pub unsafe fn new() -> Self {
        let addr = Port::new(0x0040);
        Pit {
            alarm: OptionalCell::empty(),
            client: OptionalCell::empty(),
            now: Cell::new(0),
            // Safety:
            // 1. `addr` is the location of the PIT on this chip.
            // 2. `ports` is defined correctly.
            // 3. The caller guarantees that there is no other `Pit` alive, and
            //    I/O port races to the PIT cannot cause UB.
            ports: unsafe { ports::Real::new(addr) },
        }
    }
}

impl<const R: u16, P: ports::Interface> Pit<'_, R, P> {
    /// Configures the PIT to start generating periodic interrupts.
    pub fn start(&self) {
        // Safety assumptions:
        // * We are currently running with I/O privileges
        // * There is an actual PIT device at the exected I/O addresses and not something else
        // * Interrupts will be handled properly
        // * Nobody else is accessing the PIT at the same time (shouldn't be possible unless the
        //   caller disregards the "Safety" section of `Pit::new`)

        // Set mode 2 and program reload value
        let mut pit_mcr = LocalRegisterCopy::<u8, PIT_MCR::Register>::new(0);
        pit_mcr.modify(PIT_MCR::MODE::M2 + PIT_MCR::ACCESS::LOHI + PIT_MCR::CHANNEL::C0);

        self.ports.mcr().set(pit_mcr.get());
        self.ports.cd0().set(R as u8);
        self.ports.cd0().set((R >> 8) as u8);
    }

    /// Handler to call when a PIT interrupt occurs.
    ///
    /// This will increment the internal in-memory timer and dispatch any alarms.
    pub fn handle_interrupt(&self) {
        self.now.increment();
        let now = self.now.get() as u32;

        self.alarm.take().map(|alarm| {
            if now >= alarm {
                // Alarm has elapsed, so signal the client
                self.client.map(|client| client.alarm());
            } else {
                // Alarm is still pending, check back later
                self.alarm.replace(alarm);
            }
        });
    }
}

impl<const R: u16, P: ports::Interface> Time for Pit<'_, R, P> {
    type Frequency = PitFreq<R>;

    type Ticks = Ticks32;
    fn now(&self) -> Self::Ticks {
        let now = self.now.get();
        (now as u32).into()
    }
}

impl<'a, const R: u16, P: ports::Interface> Alarm<'a> for Pit<'a, R, P> {
    fn set_alarm_client(&self, client: &'a dyn AlarmClient) {
        self.client.set(client);
    }

    fn set_alarm(&self, reference: Self::Ticks, dt: Self::Ticks) {
        self.alarm.replace(reference.into_u32() + dt.into_u32());
    }

    fn get_alarm(&self) -> Self::Ticks {
        self.alarm.map_or(0, |a| a).into()
    }

    fn disarm(&self) -> Result<(), ErrorCode> {
        self.alarm.take();
        Ok(())
    }

    fn is_armed(&self) -> bool {
        self.alarm.is_some()
    }

    fn minimum_dt(&self) -> Self::Ticks {
        1.into()
    }
}
