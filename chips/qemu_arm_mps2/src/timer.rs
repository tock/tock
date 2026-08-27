// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! ARM CMSDK APB Timer, as found on the MPS2 AN385/AN386 FPGA images.
//!
//! This is a plain 32-bit down-counter with a reload register: it counts
//! down from `VALUE` to 0 at `PCLK`, then (if enabled) reloads `VALUE` from
//! `RELOAD` and continues, raising an interrupt on every 1-to-0 transition.
//! There is no separate free-running counter or compare register.
//!
//! To expose this as a [`hil::time::Alarm`] we keep `RELOAD` fixed at
//! `u32::MAX` so the hardware free-runs and `now()` is simply
//! `u32::MAX - VALUE` (matching [`Ticks32`]'s own wraparound at 2**32).
//!
//! Arming an alarm shortens the *current* countdown by writing `VALUE`
//! directly (which, per the hardware, does not disturb `RELOAD`), so the
//! timer keeps free-running normally once the shortened countdown expires.
//!
//! Because both a deliberately-armed deadline and a routine free-run wrap
//! look identical to the hardware (both are just "`VALUE` hit 0"), every
//! interrupt resynchronizes our tracked epoch the same way, and only
//! invokes the alarm callback if an alarm was actually pending.

use core::cell::Cell;

use kernel::hil::time::{Alarm, AlarmClient, Frequency, Ticks, Ticks32, Time};
use kernel::utilities::StaticRef;
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{ReadWrite, register_bitfields};

pub const TIMER0_BASE: StaticRef<TimerRegisters> =
    unsafe { StaticRef::new(0x4000_0000 as *const TimerRegisters) };
pub const TIMER1_BASE: StaticRef<TimerRegisters> =
    unsafe { StaticRef::new(0x4000_1000 as *const TimerRegisters) };

#[repr(C)]
pub struct TimerRegisters {
    ctrl: ReadWrite<u32, CTRL::Register>,
    value: ReadWrite<u32, VALUE::Register>,
    reload: ReadWrite<u32, VALUE::Register>,
    intstatus: ReadWrite<u32, INTSTATUS::Register>,
}

register_bitfields![u32,
    CTRL [
        En OFFSET(0) NUMBITS(1) [],
        IrqEn OFFSET(3) NUMBITS(1) [],
    ],
    VALUE [
        Value OFFSET(0) NUMBITS(32) [],
    ],
    INTSTATUS [
        Irq OFFSET(0) NUMBITS(1) [],
    ],
];

pub struct Freq25MHz;
impl Frequency for Freq25MHz {
    fn frequency() -> u32 {
        crate::SYSCLK_FRQ
    }
}

pub struct Timer<'a> {
    registers: StaticRef<TimerRegisters>,
    client: kernel::utilities::cells::OptionalCell<&'a dyn AlarmClient>,
    /// Absolute tick value as of the last time we reconciled it with the
    /// live hardware `VALUE`.
    synced_now: Cell<u32>,
    /// The raw `VALUE` the hardware held at `synced_now`.
    synced_value: Cell<u32>,
    /// The most recently requested alarm target, valid regardless of
    /// whether it is still pending (per the `Alarm::get_alarm` contract).
    target: Cell<u32>,
    /// Whether the next "VALUE hit 0" interrupt corresponds to an
    /// actual armed alarm, as opposed to a routine free-run wrap.
    armed: Cell<bool>,
}

impl<'a> Timer<'a> {
    pub fn new(registers: StaticRef<TimerRegisters>) -> Timer<'a> {
        registers.ctrl.set(0);
        registers.reload.write(VALUE::Value.val(u32::MAX));
        registers.value.write(VALUE::Value.val(u32::MAX));
        registers.ctrl.write(CTRL::En::SET + CTRL::IrqEn::SET);

        Timer {
            registers,
            client: kernel::utilities::cells::OptionalCell::empty(),
            synced_now: Cell::new(0),
            synced_value: Cell::new(u32::MAX),
            target: Cell::new(0),
            armed: Cell::new(false),
        }
    }

    /// The absolute tick count that a raw `VALUE` reading corresponds to.
    ///
    /// Exact modulo 2**32 no matter how many reloads have happened since the
    /// last resync: a full countdown is `u32::MAX` ticks down to 0 plus one
    /// more to reload, i.e. exactly 2**32 ticks, which is invisible in
    /// [`Ticks32`] arithmetic. So the wrapping subtraction below recovers the
    /// elapsed time whether or not `VALUE` has wrapped in between.
    fn now_from(&self, value: u32) -> u32 {
        self.synced_now
            .get()
            .wrapping_add(self.synced_value.get().wrapping_sub(value))
    }

    /// Re-anchor the tracked epoch onto the live `VALUE`.
    fn resync(&self) {
        let value = self.registers.value.read(VALUE::Value);
        self.synced_now.set(self.now_from(value));
        self.synced_value.set(value);
    }

    pub fn handle_interrupt(&self) {
        self.registers.intstatus.write(INTSTATUS::Irq::SET);

        // Re-anchor on what VALUE actually reads, rather than assuming that
        // exactly `synced_value` ticks elapsed and that the hardware has
        // since reloaded RELOAD (u32::MAX). That assumption loses one tick
        // per interrupt -- it accounts for the countdown to 0 but not for
        // the extra tick the reload itself takes -- and the loss accumulates
        // over every alarm expiry and every free-run wrap.
        self.resync();

        if self.armed.take() {
            self.client.map(|client| client.alarm());
        }
    }
}

impl Time for Timer<'_> {
    type Frequency = Freq25MHz;
    type Ticks = Ticks32;

    fn now(&self) -> Ticks32 {
        Ticks32::from(self.now_from(self.registers.value.read(VALUE::Value)))
    }
}

impl<'a> Alarm<'a> for Timer<'a> {
    fn set_alarm_client(&self, client: &'a dyn AlarmClient) {
        self.client.set(client);
    }

    fn set_alarm(&self, reference: Ticks32, dt: Ticks32) {
        let now = self.now();
        let target = reference.wrapping_add(dt);
        self.target.set(target.into_u32());

        let elapsed = now.wrapping_sub(reference);
        let remaining = if elapsed >= dt {
            self.minimum_dt()
        } else {
            let r = dt.wrapping_sub(elapsed);
            if r < self.minimum_dt() {
                self.minimum_dt()
            } else {
                r
            }
        };

        // Resync to "now" before shortening the countdown: the write to
        // VALUE below becomes the new reference point for `now_from`.
        self.synced_now.set(now.into_u32());
        let remaining_raw = remaining.into_u32();
        self.synced_value.set(remaining_raw);
        self.armed.set(true);
        self.registers.value.write(VALUE::Value.val(remaining_raw));
    }

    fn get_alarm(&self) -> Ticks32 {
        Ticks32::from(self.target.get())
    }

    fn disarm(&self) -> Result<(), kernel::ErrorCode> {
        // The hardware will still raise an interrupt when the shortened
        // countdown reaches 0, but with `armed` cleared that interrupt is
        // treated as an ordinary free-run wrap and no callback fires.
        self.armed.set(false);
        Ok(())
    }

    fn is_armed(&self) -> bool {
        self.armed.get()
    }

    fn minimum_dt(&self) -> Ticks32 {
        // Arbitrary, but high enough to be comfortably more than needed (QEMU
        // doesn't actually cap CPU execution speed, but the timer is wall
        // time, so ten ticks (@25 MHz) is likely a few hundred instructions).
        Ticks32::from(10)
    }
}
