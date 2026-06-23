// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Timer driver for the CMSDK timer on Musca-B1 located in its SSE-200 subsystem.
//! The timer is not reliable in qemu. Some times it runs faster sometimes slower...

use kernel::hil;
use kernel::hil::time::{Alarm, Ticks, Ticks32, Time};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

register_structs! {
    /// Timer 0
    TimerRegisters {
        /// Control Register
        (0x000 => ctrl: ReadWrite<u32, CTRL::Register>),
        /// Current Timer Counter Value
        (0x004 => value: ReadWrite<u32, VALUE::Register>),
        /// Counter Reload Value
        (0x008 => reload: ReadWrite<u32, RELOAD::Register>),
        /// Timer Interrupt status register and clear register
        (0x00C => intstatus_clear: ReadWrite<u32, INTSTATUS::Register>),
        (0x010 => @END),
    }
}
register_bitfields![u32,
CTRL [
    /// Enable
    ENABLE OFFSET(0) NUMBITS(1) [
        /// Timer is disabled
        TimerIsDisabled = 0,
        /// Timer is enabled
        TimerIsEnabled = 1
    ],
    /// External Input as Enable
    EXTIN OFFSET(1) NUMBITS(1) [
        /// External Input as Enable is disabled
        ExternalInputAsEnableIsDisabled = 0,
        /// External Input as Enable is enabled
        ExternalInputAsEnableIsEnabled = 1
    ],
    /// External Clock Enable
    EXTCLK OFFSET(2) NUMBITS(1) [
        /// External Clock is disabled
        ExternalClockIsDisabled = 0,
        /// External Clock is enabled
        ExternalClockIsEnabled = 1
    ],
    /// Interrupt Enable
    INTEN OFFSET(3) NUMBITS(1) [
        /// Interrupt is disabled
        InterruptIsDisabled = 0,
        /// Interrupt is enabled
        InterruptIsEnabled = 1
    ]
],
VALUE [
    VALUE OFFSET (0) NUMBITS (32) []
],
RELOAD [
    VALUE OFFSET (0) NUMBITS (32) []
],
INTSTATUS [
    VALUE OFFSET (0) NUMBITS (32) []
],
INTCLEAR [
    VALUE OFFSET (0) NUMBITS (32) []
]
];
const TIMER0_BASE_SEC: StaticRef<TimerRegisters> =
    unsafe { StaticRef::new(0x5000_0000 as *const TimerRegisters) };
const TIMER0_BASE_NSEC: StaticRef<TimerRegisters> =
    unsafe { StaticRef::new(0x4000_0000 as *const TimerRegisters) };

const TIMER1_BASE_SEC: StaticRef<TimerRegisters> =
    unsafe { StaticRef::new(0x5000_1000 as *const TimerRegisters) };
const TIMER1_BASE_NSEC: StaticRef<TimerRegisters> =
    unsafe { StaticRef::new(0x4000_1000 as *const TimerRegisters) };

pub struct CMSDKTimer<'a> {
    alarm_regs: StaticRef<TimerRegisters>, // Used for triggering events
    counter_regs: StaticRef<TimerRegisters>, // Free-running for reliable now()
    client: OptionalCell<&'a dyn hil::time::AlarmClient>,
}

impl<'a> CMSDKTimer<'a> {
    /// Creates a combined timer using Timer0 for alarms and Timer1 as the steady clock
    pub const fn new_combined_sec() -> CMSDKTimer<'a> {
        CMSDKTimer {
            alarm_regs: TIMER0_BASE_SEC,
            counter_regs: TIMER1_BASE_SEC,
            client: OptionalCell::empty(),
        }
    }

    /// Creates a combined timer using Timer0 for alarms and Timer1 as the steady clock (Non-secure)
    pub const fn new_combined_nsec() -> CMSDKTimer<'a> {
        CMSDKTimer {
            alarm_regs: TIMER0_BASE_NSEC,
            counter_regs: TIMER1_BASE_NSEC,
            client: OptionalCell::empty(),
        }
    }

    pub fn start_counter(&self) {
        // Set counter to max value so it takes a long time to wrap
        self.counter_regs.reload.set(0xFFFFFFFF);
        self.counter_regs
            .ctrl
            .modify(CTRL::INTEN::InterruptIsDisabled);
        self.counter_regs.ctrl.modify(CTRL::ENABLE::TimerIsEnabled);
    }

    fn enable_alarm_interrupt(&self) {
        self.alarm_regs.ctrl.modify(CTRL::INTEN::InterruptIsEnabled);
        self.alarm_regs.ctrl.modify(CTRL::ENABLE::TimerIsEnabled);
    }

    fn disable_alarm_interrupt(&self) {
        self.alarm_regs
            .ctrl
            .modify(CTRL::INTEN::InterruptIsDisabled);
        self.alarm_regs.ctrl.modify(CTRL::ENABLE::TimerIsDisabled);
    }

    pub fn handle_interrupt(&self) {
        // Clear interrupt on the alarm timer
        self.alarm_regs.intstatus_clear.set(1);
        // Disable it so it doesn't fire repeatedly
        self.disable_alarm_interrupt();
        // Signal the client
        self.client.map(|client| client.alarm());
    }
}

impl Time for CMSDKTimer<'_> {
    type Frequency = hil::time::Freq32KHz;
    type Ticks = Ticks32;

    fn now(&self) -> Self::Ticks {
        // Reliable now() comes from the counter timer which is never reset.
        // CMSDK timers usually count down. We subtract from reload to get an increasing value.
        let reload = self.counter_regs.reload.get();
        let val = self.counter_regs.value.get();
        Self::Ticks::from(reload.wrapping_sub(val))
    }
}

impl<'a> Alarm<'a> for CMSDKTimer<'a> {
    fn set_alarm_client(&self, client: &'a dyn hil::time::AlarmClient) {
        self.client.set(client);
    }

    fn set_alarm(&self, reference: Self::Ticks, dt: Self::Ticks) {
        let now = self.now();
        let target = reference.wrapping_add(dt);
        let mut diff = target.wrapping_sub(now).into_u32();

        if reference > now || diff < self.minimum_dt().into_u32() {
            diff = self.minimum_dt().into_u32();
        }

        // Program the alarm hardware
        self.alarm_regs.reload.set(diff);
        self.enable_alarm_interrupt();
    }

    fn get_alarm(&self) -> Self::Ticks {
        // Returns the absolute time the alarm is set to fire
        let remaining = self.alarm_regs.value.get();
        self.now().wrapping_add(Ticks32::from(remaining))
    }

    fn disarm(&self) -> Result<(), ErrorCode> {
        self.disable_alarm_interrupt();
        Ok(())
    }

    fn is_armed(&self) -> bool {
        self.alarm_regs
            .ctrl
            .any_matching_bits_set(CTRL::INTEN::InterruptIsEnabled)
    }

    fn minimum_dt(&self) -> Self::Ticks {
        // TODO: not tested, arbitrary value
        Self::Ticks::from(50)
    }
}
