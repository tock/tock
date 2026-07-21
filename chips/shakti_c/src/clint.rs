// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! 64-bit CLINT (core-local interruptor) machine-timer driver for the SHAKTI
//! C-Class SoC.
//!
//! The SHAKTI `clint@2000000` is *almost* `riscv,clint0`-compatible, but its
//! `mkclint_axi4` does **not** honour the intra-register byte offset for
//! sub-64-bit reads of the `mtime` / `mtimecmp` region: the address shift in the
//! read path is commented out in RTL (`devices/clint/clint.bsv`), so a 32-bit
//! read of `mtime+4` (`0xBFFC`) returns `mtime[31:0]` — the *low* word — instead
//! of `mtime[63:32]`. The reusable `sifive::clint` driver assembles the 64-bit
//! counter from two separate 32-bit reads, so on this SoC it computes a corrupt
//! `now()` (high word mirrors the low word) and any alarm it programs is
//! unreachable.
//!
//! This driver instead accesses `mtime` and `mtimecmp` as single **64-bit**
//! (`ld`/`sd`) operations, which the SHAKTI CLINT services correctly (a 64-bit
//! access returns/sets the full counter).

use kernel::hil::time::{self, Alarm, Freq10MHz, Ticks, Ticks64, Time};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{register_structs, ReadWrite};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

register_structs! {
    pub ClintRegisters {
        /// Machine-software-interrupt-pending (hart 0). Writing 1 raises a
        /// machine software interrupt; writing 0 clears it.
        (0x0000 => msip: ReadWrite<u32>),
        (0x0004 => _reserved0),
        /// Machine time-compare (hart 0). The machine timer interrupt is pending
        /// while `mtime >= mtimecmp`. Accessed as a single 64-bit register.
        (0x4000 => mtimecmp: ReadWrite<u64>),
        (0x4008 => _reserved1),
        /// Free-running machine time counter. Accessed as a single 64-bit
        /// register (a 32-bit read of `0xBFFC` does not return the high word on
        /// this SoC; see the module docs).
        (0xBFF8 => mtime: ReadWrite<u64>),
        (0xC000 => @END),
    }
}

pub const CLINT_BASE: StaticRef<ClintRegisters> =
    unsafe { StaticRef::new(0x0200_0000 as *const ClintRegisters) };

/// SHAKTI C-Class CLINT machine timer / alarm, clocked at the SoC's 10 MHz
/// `timebase-frequency`.
pub struct Clint<'a> {
    registers: StaticRef<ClintRegisters>,
    client: OptionalCell<&'a dyn time::AlarmClient>,
}

impl Clint<'_> {
    pub fn new(base: &StaticRef<ClintRegisters>) -> Self {
        Self {
            registers: *base,
            client: OptionalCell::empty(),
        }
    }

    /// Service a machine timer interrupt: disarm the comparator (so it does not
    /// immediately re-fire) and notify the alarm client.
    pub fn handle_interrupt(&self) {
        self.disable_timer();
        self.client.map(|client| client.alarm());
    }

    /// Disarm the comparator by setting it to its maximum value.
    fn disable_timer(&self) {
        self.registers.mtimecmp.set(u64::MAX);
    }
}

impl Time for Clint<'_> {
    type Frequency = Freq10MHz;
    type Ticks = Ticks64;

    fn now(&self) -> Ticks64 {
        // Single 64-bit read: atomic, so unlike the 32-bit-halves path there is
        // no low/high tearing to compensate for.
        Ticks64::from(self.registers.mtime.get())
    }
}

impl<'a> Alarm<'a> for Clint<'a> {
    fn set_alarm_client(&self, client: &'a dyn time::AlarmClient) {
        self.client.set(client);
    }

    fn set_alarm(&self, reference: Ticks64, dt: Ticks64) {
        let now = self.now();
        let mut expire = reference.wrapping_add(dt);
        if !now.within_range(reference, expire) {
            // The requested expiration is already in the past; fire ASAP.
            expire = now;
        }
        // Single 64-bit write: atomic, so there is no spurious early fire while
        // the comparator is half-updated (the reason the 32-bit-halves driver
        // needs its two-step compare_low/high write dance).
        self.registers.mtimecmp.set(expire.into_u64());
    }

    fn get_alarm(&self) -> Ticks64 {
        Ticks64::from(self.registers.mtimecmp.get())
    }

    fn disarm(&self) -> Result<(), ErrorCode> {
        self.disable_timer();
        Ok(())
    }

    fn is_armed(&self) -> bool {
        self.registers.mtimecmp.get() != u64::MAX
    }

    fn minimum_dt(&self) -> Ticks64 {
        Ticks64::from(1u64)
    }
}
