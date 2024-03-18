// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! RISC-V Generic Machine Timer

use kernel::hil::time::{Ticks, Ticks64};
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::ReadWrite;
use kernel::ErrorCode;

#[repr(C)]
pub struct MachineTimerCompareRegister {
    low: ReadWrite<u32>,
    high: ReadWrite<u32>,
}

pub struct MachineTimer<'a> {
    compare: &'a [MachineTimerCompareRegister],
    value_low: &'a ReadWrite<u32>,
    value_high: &'a ReadWrite<u32>,
}

impl<'a> MachineTimer<'a> {
    pub const fn new(
        compare: &'a [MachineTimerCompareRegister],
        value_low: &'a ReadWrite<u32>,
        value_high: &'a ReadWrite<u32>,
    ) -> Self {
        MachineTimer {
            compare,
            value_low,
            value_high,
        }
    }

    pub fn disable_machine_timer(&self, context_id: usize) {
        self.compare[context_id].high.set(0xFFFF_FFFF);
        self.compare[context_id].low.set(0xFFFF_FFFF);
    }

    pub fn now(&self) -> Ticks64 {
        let first_low: u32 = self.value_low.get();
        let mut high: u32 = self.value_high.get();
        let second_low: u32 = self.value_low.get();

        if second_low < first_low {
            // Wraparound
            high = self.value_high.get();
        }

        Ticks64::from(((high as u64) << 32) | second_low as u64)
    }

    pub fn set_alarm(&self, context_id: usize, reference: Ticks64, dt: Ticks64) {
        // This does not handle the 64-bit wraparound case.
        // Because mtimer fires if the counter is >= the compare,
        // handling wraparound requires setting compare to the
        // maximum value, issuing a callback on the overflow client
        // if there is one, spinning until it wraps around to 0, then
        // setting the compare to the correct value.
        let regs = self;
        let now = self.now();
        let mut expire = reference.wrapping_add(dt);

        if !now.within_range(reference, expire) {
            expire = now;
        }

        let val = expire.into_u64();

        let high = (val >> 32) as u32;
        let low = (val & 0xffffffff) as u32;

        // Recommended approach for setting the two compare registers
        // (RISC-V Privileged Architectures 3.1.15) -pal 8/6/20
        regs.compare[context_id].low.set(0xFFFF_FFFF);
        regs.compare[context_id].high.set(high);
        regs.compare[context_id].low.set(low);
    }

    pub fn get_alarm(&self, context_id: usize) -> Ticks64 {
        let mut val: u64 = (self.compare[context_id].high.get() as u64) << 32;
        val |= self.compare[context_id].low.get() as u64;
        Ticks64::from(val)
    }

    pub fn disarm(&self, context_id: usize) -> Result<(), ErrorCode> {
        self.disable_machine_timer(context_id);
        Ok(())
    }

    pub fn is_armed(&self, context_id: usize) -> bool {
        // Check if mtimecmp is the max value. If it is, then we are not armed,
        // otherwise we assume we have a value set.
        self.compare[context_id].high.get() != 0xFFFF_FFFF || self.compare[context_id].low.get() != 0xFFFF_FFFF
    }

    pub fn minimum_dt(&self) -> Ticks64 {
        Ticks64::from(1u64)
    }
}
