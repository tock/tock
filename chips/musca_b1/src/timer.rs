// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

use kernel::hil;
use kernel::hil::time::{Alarm, Ticks, Ticks32, Time};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

register_structs! {
    /// General-Purpose Timer
    GptimerRegisters {
        /// Control Reset Register
        (0x000 => gptreset: ReadOnly<u32>),
        /// Masked interrupt status register
        (0x004 => gptintm: ReadWrite<u32>),
        /// Interrupt clear register
        (0x008 => gptintc: ReadWrite<u32>),
        (0x00C => _reserved0),
        /// ALARM0 data value register
        (0x010 => gptalarm0: ReadWrite<u32>),
        /// ALARM1 data value register
        (0x014 => gptalarm1: ReadWrite<u32>),
        /// Raw interrupt status register
        (0x018 => gptintr: ReadOnly<u32>),
        /// Counter data value register
        (0x01C => gptcounter: ReadOnly<u32>),
        (0x020 => @END),
    }
}
register_bitfields![u32,
GPTRESET [
    /// CPU0 interrupt status
    GPTRESET OFFSET(0) NUMBITS(2) []
],
GPTINTM [
    /// Current masked status of the interrupt
    GPTINTM OFFSET(0) NUMBITS(2) []
],
GPTINTC [
    /// Writing 0b1 disables the ALARM[n] interrupt
    GPTINTC OFFSET(0) NUMBITS(2) []
],
GPTALARM0 [
    /// Value that triggers the ALARM0 interrupt when the counter reaches that value
     GPTALARM0_DATA OFFSET(0) NUMBITS(32) []
],
GPTALARM1 [
    /// Value that triggers the ALARM1 interrupt when the counter reaches that value
     GPTALARM1_DATA OFFSET(0) NUMBITS(32) []
],
GPTINTR [
    /// Raw interrupt state, before masking of GPTINTR interrupt
    GPTINTR OFFSET(0) NUMBITS(3) []
],
GPTCOUNTER [
    /// Current value of 32-bit Timer Counter
    GPTCOUNTER OFFSET(0) NUMBITS(32) []
]
];

const GPTIMER_BASE_NSEC: StaticRef<GptimerRegisters> =
    unsafe { StaticRef::new(0x4010C000 as *const GptimerRegisters) };
const GPTIMER_BASE_SEC: StaticRef<GptimerRegisters> =
    unsafe { StaticRef::new(0x5010C000 as *const GptimerRegisters) };

pub struct GPTimer<'a> {
    registers: StaticRef<GptimerRegisters>,
    client: OptionalCell<&'a dyn hil::time::AlarmClient>,
}

impl<'a> GPTimer<'a> {
    pub const fn new_sec() -> GPTimer<'a> {
        GPTimer {
            registers: GPTIMER_BASE_SEC,
            client: OptionalCell::empty(),
        }
    }

    pub const fn new_nsec() -> GPTimer<'a> {
        GPTimer {
            registers: GPTIMER_BASE_NSEC,
            client: OptionalCell::empty(),
        }
    }

    fn clear_interrupt(&self) {
        self.registers.gptintc.set(0b01);
    }

    pub fn handle_interrupt(&self) {
        self.clear_interrupt();
        self.client.map(|client| client.alarm());
    }
}

impl Time for GPTimer<'_> {
    type Frequency = hil::time::Freq32KHz;
    type Ticks = Ticks32;

    fn now(&self) -> Self::Ticks {
        // Read the current counter value
        Ticks32::from(self.registers.gptcounter.get())
    }
}

impl<'a> Alarm<'a> for GPTimer<'a> {
    fn set_alarm_client(&self, client: &'a dyn hil::time::AlarmClient) {
        self.client.set(client);
    }

    fn set_alarm(&self, reference: Self::Ticks, dt: Self::Ticks) {
        let now = self.now();
        let mut expire = reference.wrapping_add(dt);

        // Ensure the expiration time isn't in the past
        if !now.within_range(reference, expire) {
            expire = now;
        }

        // Enforce minimum duration
        if expire.wrapping_sub(now) < self.minimum_dt() {
            expire = now.wrapping_add(self.minimum_dt());
        }

        self.registers.gptalarm0.set(expire.into_u32());

        // Enable the interrupt mask
        self.registers.gptintm.set(0b01);
    }

    fn get_alarm(&self) -> Self::Ticks {
        Ticks32::from(self.registers.gptalarm0.get())
    }

    fn disarm(&self) -> Result<(), ErrorCode> {
        // Disable the interrupt mask for ALARM0
        self.registers.gptintc.set(0b01);
        Ok(())
    }

    fn is_armed(&self) -> bool {
        // Check if the interrupt mask for ALARM0 is set
        (self.registers.gptintm.get() & 0b01) != 0
    }

    fn minimum_dt(&self) -> Self::Ticks {
        // TODO: not tested, arbitrary value
        Ticks32::from(10)
    }
}
