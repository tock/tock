// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! RISC-V Generic Machine Timer

use kernel::ErrorCode;
use kernel::hil::time::{Ticks, Ticks64};
use kernel::utilities::registers::ReadWrite;
use kernel::utilities::registers::interfaces::{Readable, Writeable};

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
