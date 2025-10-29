// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Timer driver, nRF5X-family
//!
//! The nRF51822 timer system operates off of the high frequency clock
//! (HFCLK) and provides three timers from the clock. Timer0 is tied
//! to the radio through some hard-coded peripheral linkages (e.g., there
//! are dedicated PPI connections between Timer0's compare events and
//! radio tasks, its capture tasks and radio events).
//!
//! This implementation provides a full-fledged Timer interface to
//! timers 0 and 2, and exposes Timer1 as an HIL Alarm, for a Tock
//! timer system. It may be that the Tock timer system should be ultimately
//! placed on top of the RTC (from the low frequency clock). It's currently
//! implemented this way as a demonstration that it can be and because
//! the full RTC/clock interface hasn't been finalized yet.
//!
//! This approach should be rewritten, such that the timer system uses
//! the RTC from the low frequency clock (lower power) and the scheduler
//! uses the high frequency clock.
//!
//! Authors
//! --------
//! * Philip Levis <pal@cs.stanford.edu>
//! * Date: August 18, 2016

use kernel::hil;
use kernel::hil::time::{Alarm, Ticks, Time};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, ReadWrite, WriteOnly};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

#[repr(C)]
pub struct TimerRegisters {
    /// Start Timer
    tasks_start: WriteOnly<u32, Task::Register>,
    /// Stop Timer
    tasks_stop: WriteOnly<u32, Task::Register>,
    /// Increment Timer (Counter mode only)
    tasks_count: WriteOnly<u32, Task::Register>,
    /// Clear time
    tasks_clear: WriteOnly<u32, Task::Register>,
    /// Shut down timer
    tasks_shutdown: WriteOnly<u32, Task::Register>,
    _reserved0: [u8; 44],
    /// Capture Timer value
    tasks_capture: [WriteOnly<u32, Task::Register>; 4],
    _reserved1: [u8; 240],
    /// Compare event
    events_compare: [ReadWrite<u32, Event::Register>; 4],
    _reserved2: [u8; 176],
    /// Shortcut register
    shorts: ReadWrite<u32, Shorts::Register>,
    _reserved3: [u8; 256],
    /// Enable interrupt
    intenset: ReadWrite<u32, Inte::Register>,
    /// Disable interrupt
    intenclr: ReadWrite<u32, Inte::Register>,
    _reserved4: [u8; 504],
    /// Timer mode selection
    mode: ReadWrite<u32>,
    /// Configure the number of bits used by the TIMER
    bitmode: ReadWrite<u32, Bitmode::Register>,
    _reserved5: [u8; 4],
    /// Timer prescaler register
    prescaler: ReadWrite<u32>,
    _reserved6: [u8; 44],
    /// Capture/Compare
    cc: [ReadWrite<u32, CC::Register>; 4],
}

register_bitfields![u32,
    Shorts [
        /// Shortcut between EVENTS_COMPARE\[0\] event and TASKS_CLEAR task
        COMPARE0_CLEAR OFFSET(0) NUMBITS(1) [
            /// Disable shortcut
            DisableShortcut = 0,
            /// Enable shortcut
            EnableShortcut = 1
        ],
        /// Shortcut between EVENTS_COMPARE\[1\] event and TASKS_CLEAR task
        COMPARE1_CLEAR OFFSET(1) NUMBITS(1) [
            /// Disable shortcut
            DisableShortcut = 0,
            /// Enable shortcut
            EnableShortcut = 1
        ],
        /// Shortcut between EVENTS_COMPARE\[2\] event and TASKS_CLEAR task
        COMPARE2_CLEAR OFFSET(2) NUMBITS(1) [
            /// Disable shortcut
            DisableShortcut = 0,
            /// Enable shortcut
            EnableShortcut = 1
        ],
        /// Shortcut between EVENTS_COMPARE\[3\] event and TASKS_CLEAR task
        COMPARE3_CLEAR OFFSET(3) NUMBITS(1) [
            /// Disable shortcut
            DisableShortcut = 0,
            /// Enable shortcut
            EnableShortcut = 1
        ],
        /// Shortcut between EVENTS_COMPARE\[4\] event and TASKS_CLEAR task
        COMPARE4_CLEAR OFFSET(4) NUMBITS(1) [
            /// Disable shortcut
            DisableShortcut = 0,
            /// Enable shortcut
            EnableShortcut = 1
        ],
        /// Shortcut between EVENTS_COMPARE\[5\] event and TASKS_CLEAR task
        COMPARE5_CLEAR OFFSET(5) NUMBITS(1) [
            /// Disable shortcut
            DisableShortcut = 0,
            /// Enable shortcut
            EnableShortcut = 1
        ],
        /// Shortcut between EVENTS_COMPARE\[0\] event and TASKS_STOP task
        COMPARE0_STOP OFFSET(8) NUMBITS(1) [
            /// Disable shortcut
            DisableShortcut = 0,
            /// Enable shortcut
            EnableShortcut = 1
        ],
        /// Shortcut between EVENTS_COMPARE\[1\] event and TASKS_STOP task
        COMPARE1_STOP OFFSET(9) NUMBITS(1) [
            /// Disable shortcut
            DisableShortcut = 0,
            /// Enable shortcut
            EnableShortcut = 1
        ],
        /// Shortcut between EVENTS_COMPARE\[2\] event and TASKS_STOP task
        COMPARE2_STOP OFFSET(10) NUMBITS(1) [
            /// Disable shortcut
            DisableShortcut = 0,
            /// Enable shortcut
            EnableShortcut = 1
        ],
        /// Shortcut between EVENTS_COMPARE\[3\] event and TASKS_STOP task
        COMPARE3_STOP OFFSET(11) NUMBITS(1) [
            /// Disable shortcut
            DisableShortcut = 0,
            /// Enable shortcut
            EnableShortcut = 1
        ],
        /// Shortcut between EVENTS_COMPARE\[4\] event and TASKS_STOP task
        COMPARE4_STOP OFFSET(12) NUMBITS(1) [
            /// Disable shortcut
            DisableShortcut = 0,
            /// Enable shortcut
            EnableShortcut = 1
        ],
        /// Shortcut between EVENTS_COMPARE\[5\] event and TASKS_STOP task
        COMPARE5_STOP OFFSET(13) NUMBITS(1) [
            /// Disable shortcut
            DisableShortcut = 0,
            /// Enable shortcut
            EnableShortcut = 1
        ]
    ],
    Inte [
        /// Write '1' to Enable interrupt on EVENTS_COMPARE\[0\] event
        COMPARE0 16,
        /// Write '1' to Enable interrupt on EVENTS_COMPARE\[1\] event
        COMPARE1 17,
        /// Write '1' to Enable interrupt on EVENTS_COMPARE\[2\] event
        COMPARE2 18,
        /// Write '1' to Enable interrupt on EVENTS_COMPARE\[3\] event
        COMPARE3 19,
        /// Write '1' to Enable interrupt on EVENTS_COMPARE\[4\] event
        COMPARE4 20,
        /// Write '1' to Enable interrupt on EVENTS_COMPARE\[5\] event
        COMPARE5 21
    ],
    Bitmode [
        /// Timer bit width
        BITMODE OFFSET(0) NUMBITS(2) [
            Bit16 = 0,
            Bit08 = 1,
            Bit24 = 2,
            Bit32 = 3
        ]
    ],
    Task [
        ENABLE 0
    ],
    Event [
        READY 0
    ],
    CC [
        CC OFFSET(0) NUMBITS(32)
    ]
];

pub enum BitmodeValue {
    Size16Bits = 0,
    Size8Bits = 1,
    Size24Bits = 2,
    Size32Bits = 3,
}

pub trait CompareClient {
    /// Passes a bitmask of which of the 4 compares/captures fired (0x0-0xf).
    fn compare(&self, bitmask: u8);
}

pub struct Timer {
    registers: StaticRef<TimerRegisters>,
    client: OptionalCell<&'static dyn CompareClient>,
}

impl Timer {
    pub const fn new(registers: StaticRef<TimerRegisters>) -> Timer {
        Timer {
            registers,
            client: OptionalCell::empty(),
        }
    }

    pub fn set_client(&self, client: &'static dyn CompareClient) {
        self.client.set(client);
    }

    /// When an interrupt occurs, check if any of the 4 compares have
    /// created an event, and if so, add it to the bitmask of triggered
    /// events that is passed to the client.

    pub fn handle_interrupt(&self) {
        self.client.map(|client| {
            let mut val = 0;
            // For each of 4 possible compare events, if it's happened,
            // clear it and store its bit in val to pass in callback.
            for i in 0..4 {
                if self.registers.events_compare[i].is_set(Event::READY) {
                    val |= 1 << i;
                    self.registers.events_compare[i].write(Event::READY::CLEAR);
                    // Disable corresponding interrupt
                    let interrupt_bit = match i {
                        0 => Inte::COMPARE0::SET,
                        1 => Inte::COMPARE1::SET,
                        2 => Inte::COMPARE2::SET,
                        3 => Inte::COMPARE3::SET,
                        4 => Inte::COMPARE4::SET,
                        _ => Inte::COMPARE5::SET,
                    };
                    self.registers.intenclr.write(interrupt_bit);
                }
            }
            client.compare(val as u8);
        });
    }
}

pub struct TimerAlarm<'a> {
    registers: StaticRef<TimerRegisters>,
    client: OptionalCell<&'a dyn hil::time::AlarmClient>,
}

// CC0 is used for capture
// CC1 is used for compare/interrupts
const CC_CAPTURE: usize = 0;
const CC_COMPARE: usize = 1;

impl<'a> TimerAlarm<'a> {
    pub const fn new(registers: StaticRef<TimerRegisters>) -> TimerAlarm<'a> {
        TimerAlarm {
            registers,
            client: OptionalCell::empty(),
        }
    }

    fn clear_alarm(&self) {
        self.registers.events_compare[CC_COMPARE].write(Event::READY::CLEAR);
        self.registers.tasks_stop.write(Task::ENABLE::SET);
        self.registers.tasks_clear.write(Task::ENABLE::SET);
        self.disable_interrupts();
    }

    pub fn handle_interrupt(&self) {
        self.clear_alarm();
        self.client.map(|client| {
            client.alarm();
        });
    }

    fn enable_interrupts(&self) {
        self.registers.intenset.write(Inte::COMPARE1::SET);
    }

    fn disable_interrupts(&self) {
        self.registers.intenclr.write(Inte::COMPARE1::SET);
    }

    fn interrupts_enabled(&self) -> bool {
        self.registers.intenset.is_set(Inte::COMPARE1)
    }

    fn value(&self) -> u32 {
        self.registers.tasks_capture[CC_CAPTURE].write(Task::ENABLE::SET);
        self.registers.cc[CC_CAPTURE].get()
    }
}

impl Time for TimerAlarm<'_> {
    type Frequency = hil::time::Freq16KHz;
    // Note: we always use BITMODE::32.
    type Ticks = hil::time::Ticks32;

    fn now(&self) -> Self::Ticks {
        Self::Ticks::from(self.value())
    }
}

impl<'a> Alarm<'a> for TimerAlarm<'a> {
    fn set_alarm_client(&self, client: &'a dyn hil::time::AlarmClient) {
        self.client.set(client);
    }

    fn set_alarm(&self, reference: Self::Ticks, dt: Self::Ticks) {
        self.disable_interrupts();

        const SYNC_TICS: u32 = 2;
        let regs = &*self.registers;

        let mut expire = reference.wrapping_add(dt);

        let now = self.now();
        let earliest_possible = now.wrapping_add(Self::Ticks::from(SYNC_TICS));

        if !now.within_range(reference, expire) || expire.wrapping_sub(now).into_u32() <= SYNC_TICS
        {
            expire = earliest_possible;
        }

        regs.bitmode.write(Bitmode::BITMODE::Bit32);
        regs.cc[CC_COMPARE].write(CC::CC.val(expire.into_u32()));
        regs.tasks_start.write(Task::ENABLE::SET);
        self.enable_interrupts();
    }

    fn get_alarm(&self) -> Self::Ticks {
        Self::Ticks::from(self.registers.cc[CC_COMPARE].read(CC::CC))
    }

    fn disarm(&self) -> Result<(), ErrorCode> {
        self.disable_interrupts();
        Ok(())
    }

    fn is_armed(&self) -> bool {
        self.interrupts_enabled()
    }

    fn minimum_dt(&self) -> Self::Ticks {
        // TODO: not tested, arbitrary value
        Self::Ticks::from(10)
    }
}
