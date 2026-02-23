// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

use cortexm33::support::with_interrupts_disabled;
use kernel::hil;
use kernel::hil::time::{Alarm, Ticks, Ticks32, Time};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
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
    registers: StaticRef<TimerRegisters>,
    client: OptionalCell<&'a dyn hil::time::AlarmClient>,
}

impl<'a> CMSDKTimer<'a> {
    pub const fn new_timer0_sec() -> CMSDKTimer<'a> {
        CMSDKTimer {
            registers: TIMER0_BASE_SEC,
            client: OptionalCell::empty(),
        }
    }

    fn enable_interrupt0(&self) {
        self.registers.ctrl.modify(CTRL::ENABLE::TimerIsEnabled);
    }

    fn disable_interrupt0(&self) {
        self.registers.ctrl.modify(CTRL::ENABLE::TimerIsDisabled);
    }

    pub fn handle_interrupt(&self) {
        self.registers.intstatus_clear.set(1);
        self.client.map(|client| client.alarm());
    }
}

impl Time for CMSDKTimer<'_> {
    type Frequency = hil::time::Freq1MHz;
    type Ticks = Ticks32;

    fn now(&self) -> Self::Ticks {
        Self::Ticks::from(self.registers.value.get())
    }
}

impl<'a> Alarm<'a> for CMSDKTimer<'a> {
    fn set_alarm_client(&self, client: &'a dyn hil::time::AlarmClient) {
        self.client.set(client);
    }

    fn set_alarm(&self, reference: Self::Ticks, dt: Self::Ticks) {
        let mut expire = reference.wrapping_add(dt);
        let now = self.now();
        if !now.within_range(reference, expire) {
            expire = now;
        }

        if expire.wrapping_sub(now) < self.minimum_dt() {
            expire = now.wrapping_add(self.minimum_dt());
        }

        self.registers.value.set(expire.into_u32());
        self.enable_interrupt0();
    }

    fn get_alarm(&self) -> Self::Ticks {
        Self::Ticks::from(self.registers.value.get())
    }

    fn disarm(&self) -> Result<(), ErrorCode> {
        self.registers.ctrl.modify(CTRL::ENABLE::TimerIsDisabled);
        unsafe {
            with_interrupts_disabled(|| {
                // Clear pending interrupts
                cortexm33::nvic::Nvic::new(3).clear_pending();
            });
        }
        self.disable_interrupt0();
        Ok(())
    }

    fn is_armed(&self) -> bool {
        let armed = self
            .registers
            .ctrl
            .any_matching_bits_set(CTRL::ENABLE::TimerIsEnabled);
        if armed {
            return true;
        }
        false
    }

    fn minimum_dt(&self) -> Self::Ticks {
        Self::Ticks::from(50) // todo
    }
}
