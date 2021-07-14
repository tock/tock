//! RISC-V Generic Machine Timer

use kernel::hil::time::{Ticks, Ticks64};
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::ReadWrite;
use kernel::ErrorCode;

pub struct MachineTimer<'a> {
    compare_low: &'a ReadWrite<u32>,
    compare_high: &'a ReadWrite<u32>,
    value_low: &'a ReadWrite<u32>,
    value_high: &'a ReadWrite<u32>,
}

impl<'a> MachineTimer<'a> {
    pub const fn new(
        compare_low: &'a ReadWrite<u32>,
        compare_high: &'a ReadWrite<u32>,
        value_low: &'a ReadWrite<u32>,
        value_high: &'a ReadWrite<u32>,
    ) -> Self {
        MachineTimer {
            compare_low,
            compare_high,
            value_low,
            value_high,
        }
    }

    pub fn disable_machine_timer(&self) {
        self.compare_high.set(0xFFFF_FFFF);
        self.compare_low.set(0xFFFF_FFFF);
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

    pub fn set_alarm(&self, reference: Ticks64, dt: Ticks64) {
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
        regs.compare_low.set(0xFFFF_FFFF);
        regs.compare_high.set(high);
        regs.compare_low.set(low);
    }

    pub fn get_alarm(&self) -> Ticks64 {
        let mut val: u64 = (self.compare_high.get() as u64) << 32;
        val |= self.compare_low.get() as u64;
        Ticks64::from(val)
    }

    pub fn disarm(&self) -> Result<(), ErrorCode> {
        self.disable_machine_timer();
        Ok(())
    }

    pub fn is_armed(&self) -> bool {
        // Check if mtimecmp is the max value. If it is, then we are not armed,
        // otherwise we assume we have a value set.
        self.compare_high.get() != 0xFFFF_FFFF || self.compare_low.get() != 0xFFFF_FFFF
    }

    pub fn minimum_dt(&self) -> Ticks64 {
        Ticks64::from(1u64)
    }
}
