// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! RISC-V Generic Machine Timer

use kernel::ErrorCode;
use kernel::hil::time::{Ticks, Ticks64};
use kernel::utilities::registers::ReadWrite;
use kernel::utilities::registers::interfaces::{Readable, Writeable};

/// A RISC-V machine timer, read/written as a single 64-bit MMIO access.
///
/// # Atomicity assumption
///
/// This driver assumes `compare` and `value` are backed by a genuinely
/// 64-bit-wide MMIO register — i.e. that a single `u64` load/store to
/// them is an atomic bus transaction, per RISC-V Privileged Architectures
/// §3.1.15 ("Attempts to read the mtime register while an update is
/// in progress do not cause the read to stall ... implementations must
/// provide the appearance that writes to mtimecmp ... are atomic when
/// naturally aligned").
///
/// This holds for a standard/reference CLINT (e.g. SiFive-style), which is
/// what every RV64 board Tock currently supports uses. It is NOT universal:
/// at least one shipping RV64 core (T-Head C9xx, used in Allwinner D1 /
/// Sipeed Lichee RV / Nezha) only supports 32-bit-wide mtime/mtimecmp
/// accesses and requires split reads/writes
/// (see <https://github.com/riscv/riscv-isa-manual/issues/639> and the
/// upstream Linux `timer-clint` T-Head quirk driver). If a board ever targets
/// such a core, it must use the 32-bit driver instead for correctness.
pub struct MachineTimer<'a> {
    compare: &'a ReadWrite<u64>,
    value: &'a ReadWrite<u64>,
}

impl<'a> MachineTimer<'a> {
    pub const fn new(compare: &'a ReadWrite<u64>, value: &'a ReadWrite<u64>) -> Self {
        MachineTimer { compare, value }
    }

    pub fn disable_machine_timer(&self) {
        self.compare.set(0xFFFF_FFFF_FFFF_FFFF);
    }

    pub fn now(&self) -> Ticks64 {
        Ticks64::from(self.value.get())
    }

    pub fn set_alarm(&self, reference: Ticks64, dt: Ticks64) {
        // This does not handle the 64-bit wraparound case.
        // Because mtimer fires if the counter is >= the compare,
        // handling wraparound requires setting compare to the
        // maximum value, issuing a callback on the overflow client
        // if there is one, spinning until it wraps around to 0, then
        // setting the compare to the correct value.
        let now = self.now();
        let mut expire = reference.wrapping_add(dt);

        if !now.within_range(reference, expire) {
            expire = now;
        }

        self.compare.set(expire.into_u64());
    }

    pub fn get_alarm(&self) -> Ticks64 {
        Ticks64::from(self.compare.get())
    }

    pub fn disarm(&self) -> Result<(), ErrorCode> {
        self.disable_machine_timer();
        Ok(())
    }

    pub fn is_armed(&self) -> bool {
        // Check if mtimecmp is the max value. If it is, then we are not armed,
        // otherwise we assume we have a value set.
        self.compare.get() != 0xFFFF_FFFF_FFFF_FFFF
    }

    pub fn minimum_dt(&self) -> Ticks64 {
        Ticks64::from(1u64)
    }
}
