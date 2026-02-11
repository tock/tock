// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

use cortexm33::support::with_interrupts_disabled;
use kernel::hil;
use kernel::hil::time::{Alarm, Ticks, Ticks32, Time};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

use crate::interrupts::TIMER0_IRQ_0;

register_structs! {
    /// Controls time and alarms
    TimerRegisters {
        /// Write to bits 63:32 of time always write timelw before timehw
        (0x000 => timehw: ReadWrite<u32>),
        /// Write to bits 31:0 of time writes do not get copied to time until timehw is written
        (0x004 => timelw: ReadWrite<u32>),
        /// Read from bits 63:32 of time always read timelr before timehr
        (0x008 => timehr: ReadWrite<u32>),
        /// Read from bits 31:0 of time
        (0x00C => timelr: ReadWrite<u32>),
        /// Arm alarm 0, and configure the time it will fire. Once armed, the alarm fires when TIMER_ALARM0 == TIMELR. The alarm will disarm itself once it fires, and can be disarmed early using the ARMED status register.
        (0x010 => alarm0: ReadWrite<u32>),
        /// Arm alarm 1, and configure the time it will fire. Once armed, the alarm fires when TIMER_ALARM1 == TIMELR. The alarm will disarm itself once it fires, and can be disarmed early using the ARMED status register.
        (0x014 => alarm1: ReadWrite<u32>),
        /// Arm alarm 2, and configure the time it will fire. Once armed, the alarm fires when TIMER_ALARM2 == TIMELR. The alarm will disarm itself once it fires, and can be disarmed early using the ARMED status register.
        (0x018 => alarm2: ReadWrite<u32>),
        /// Arm alarm 3, and configure the time it will fire. Once armed, the alarm fires when TIMER_ALARM3 == TIMELR. The alarm will disarm itself once it fires, and can be disarmed early using the ARMED status register.
        (0x01C => alarm3: ReadWrite<u32>),
        /// Indicates the armed/disarmed status of each alarm. A write to the corresponding ALARMx register arms the alarm. Alarms automatically disarm upon firing, but writing ones here will disarm immediately without waiting to fire.
        (0x020 => armed: ReadWrite<u32>),
        /// Raw read from bits 63:32 of time (no side effects)
        (0x024 => timerawh: ReadWrite<u32>),
        /// Raw read from bits 31:0 of time (no side effects)
        (0x028 => timerawl: ReadWrite<u32>),
        /// Set bits high to enable pause when the corresponding debug ports are active
        (0x02C => dbgpause: ReadWrite<u32, DBGPAUSE::Register>),
        /// Set high to pause the timer
        (0x030 => pause: ReadWrite<u32>),
        /// Set locked bit to disable write access to timer Once set, cannot be cleared (without a reset)
        (0x034 => locked: ReadWrite<u32>),
        /// Selects the source for the timer. Defaults to the normal tick configured in the ticks block (typically configured to 1 microsecond). Writing to 1 will ignore the tick and count clk_sys cycles instead.
        (0x038 => source: ReadWrite<u32>),
        /// Raw Interrupts
        (0x03C => intr: ReadWrite<u32, INTR::Register>),
        /// Interrupt Enable
        (0x040 => inte: ReadWrite<u32, INTE::Register>),
        /// Interrupt Force
        (0x044 => intf: ReadWrite<u32, INTF::Register>),
        /// Interrupt status after masking & forcing
        (0x048 => ints: ReadWrite<u32, INTS::Register>),
        (0x04C => @END),
    }
}
register_bitfields![u32,
TIMEHW [

    TIMEHW OFFSET(0) NUMBITS(32) []
],
TIMELW [

    TIMELW OFFSET(0) NUMBITS(32) []
],
TIMEHR [

    TIMEHR OFFSET(0) NUMBITS(32) []
],
TIMELR [

    TIMELR OFFSET(0) NUMBITS(32) []
],
ALARM0 [

    ALARM0 OFFSET(0) NUMBITS(32) []
],
ALARM1 [

    ALARM1 OFFSET(0) NUMBITS(32) []
],
ALARM2 [

    ALARM2 OFFSET(0) NUMBITS(32) []
],
ALARM3 [

    ALARM3 OFFSET(0) NUMBITS(32) []
],
ARMED [

    ARMED OFFSET(0) NUMBITS(4) []
],
TIMERAWH [

    TIMERAWH OFFSET(0) NUMBITS(32) []
],
TIMERAWL [

    TIMERAWL OFFSET(0) NUMBITS(32) []
],
DBGPAUSE [
    /// Pause when processor 1 is in debug mode
    DBG1 OFFSET(2) NUMBITS(1) [],
    /// Pause when processor 0 is in debug mode
    DBG0 OFFSET(1) NUMBITS(1) []
],
PAUSE [

    PAUSE OFFSET(0) NUMBITS(1) []
],
LOCKED [

    LOCKED OFFSET(0) NUMBITS(1) []
],
SOURCE [

    CLK_SYS OFFSET(0) NUMBITS(1) [

        TICK = 0
    ]
],
INTR [

    ALARM_3 OFFSET(3) NUMBITS(1) [],

    ALARM_2 OFFSET(2) NUMBITS(1) [],

    ALARM_1 OFFSET(1) NUMBITS(1) [],

    ALARM_0 OFFSET(0) NUMBITS(1) []
],
INTE [

    ALARM_3 OFFSET(3) NUMBITS(1) [],

    ALARM_2 OFFSET(2) NUMBITS(1) [],

    ALARM_1 OFFSET(1) NUMBITS(1) [],

    ALARM_0 OFFSET(0) NUMBITS(1) []
],
INTF [

    ALARM_3 OFFSET(3) NUMBITS(1) [],

    ALARM_2 OFFSET(2) NUMBITS(1) [],

    ALARM_1 OFFSET(1) NUMBITS(1) [],

    ALARM_0 OFFSET(0) NUMBITS(1) []
],
INTS [

    ALARM_3 OFFSET(3) NUMBITS(1) [],

    ALARM_2 OFFSET(2) NUMBITS(1) [],

    ALARM_1 OFFSET(1) NUMBITS(1) [],

    ALARM_0 OFFSET(0) NUMBITS(1) []
]
];

const TIMER0_BASE: StaticRef<TimerRegisters> =
    unsafe { StaticRef::new(0x400B0000 as *const TimerRegisters) };

pub struct RPTimer<'a> {
    registers: StaticRef<TimerRegisters>,
    client: OptionalCell<&'a dyn hil::time::AlarmClient>,
}

impl<'a> RPTimer<'a> {
    pub const fn new_timer0() -> RPTimer<'a> {
        RPTimer {
            registers: TIMER0_BASE,
            client: OptionalCell::empty(),
        }
    }

    fn enable_interrupt0(&self) {
        self.registers.inte.modify(INTE::ALARM_0::SET);
    }

    fn disable_interrupt0(&self) {
        self.registers.inte.modify(INTE::ALARM_0::CLEAR);
    }

    fn enable_timer_interrupt0(&self) {
        // Even though setting the INTE::ALARM_0 bit should be enough to enable
        // the interrupt firing, it seems that RP2040 requires manual NVIC
        // enabling of the interrupt.
        //
        // Failing to do so results in the interrupt being set as pending but
        // not fired. This means that the interrupt will be handled whenever the
        // next kernel tasks are processed.
        unsafe {
            with_interrupts_disabled(|| {
                cortexm33::nvic::Nvic::new(TIMER0_IRQ_0).enable();
            })
        }
    }

    fn disable_timer_interrupt0(&self) {
        // Even though clearing the INTE::ALARM_0 bit should be enough to disable
        // the interrupt firing, it seems that RP2040 requires manual NVIC
        // disabling of the interrupt.
        unsafe {
            cortexm33::nvic::Nvic::new(TIMER0_IRQ_0).disable();
        }
    }

    pub fn handle_interrupt(&self) {
        self.registers.intr.modify(INTR::ALARM_0::SET);
        self.client.map(|client| client.alarm());
    }
}

impl Time for RPTimer<'_> {
    type Frequency = hil::time::Freq1MHz;
    type Ticks = Ticks32;

    fn now(&self) -> Self::Ticks {
        Self::Ticks::from(self.registers.timerawl.get())
    }
}

impl<'a> Alarm<'a> for RPTimer<'a> {
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

        self.registers.alarm0.set(expire.into_u32());
        self.enable_timer_interrupt0();
        self.enable_interrupt0();
    }

    fn get_alarm(&self) -> Self::Ticks {
        Self::Ticks::from(self.registers.alarm0.get())
    }

    fn disarm(&self) -> Result<(), ErrorCode> {
        self.registers.armed.set(1);
        unsafe {
            with_interrupts_disabled(|| {
                // Clear pending interrupts
                cortexm33::nvic::Nvic::new(TIMER0_IRQ_0).clear_pending();
            });
        }
        self.disable_interrupt0();
        self.disable_timer_interrupt0();
        Ok(())
    }

    fn is_armed(&self) -> bool {
        let armed = self.registers.armed.get() & 0b0001;
        if armed == 1 {
            return true;
        }
        false
    }

    fn minimum_dt(&self) -> Self::Ticks {
        Self::Ticks::from(50)
    }
}
