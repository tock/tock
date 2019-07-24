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

use kernel::common::cells::OptionalCell;
use kernel::common::registers::{self, register_bitfields, ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use kernel::hil;

const INSTANCES: [StaticRef<TimerRegisters>; 3] = unsafe {
    [
        StaticRef::new(0x40008000 as *const TimerRegisters),
        StaticRef::new(0x40009000 as *const TimerRegisters),
        StaticRef::new(0x4000A000 as *const TimerRegisters),
    ]
};

#[repr(C)]
struct TimerRegisters {
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

pub static mut TIMER0: TimerAlarm = TimerAlarm::new(0);
pub static mut ALARM1: TimerAlarm = TimerAlarm::new(1);
pub static mut TIMER2: Timer = Timer::new(2);

pub trait CompareClient {
    /// Passes a bitmask of which of the 4 compares/captures fired (0x0-0xf).
    fn compare(&self, bitmask: u8);
}

pub struct Timer {
    registers: StaticRef<TimerRegisters>,
    client: OptionalCell<&'static CompareClient>,
}

impl Timer {
    pub const fn new(instance: usize) -> Timer {
        Timer {
            registers: INSTANCES[instance],
            client: OptionalCell::empty(),
        }
    }

    pub fn set_client(&self, client: &'static CompareClient) {
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
                    val = val | 1 << i;
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

pub struct TimerAlarm {
    registers: StaticRef<TimerRegisters>,
    client: OptionalCell<&'static hil::time::Client>,
}

// CC0 is used for capture
// CC1 is used for compare/interrupts
const ALARM_CAPTURE: usize = 0;
const ALARM_COMPARE: usize = 1;
const ALARM_INTERRUPT_BIT: registers::Field<u32, Inte::Register> = Inte::COMPARE1;
const ALARM_INTERRUPT_BIT_SET: registers::FieldValue<u32, Inte::Register> = Inte::COMPARE1::SET;

impl TimerAlarm {
    const fn new(instance: usize) -> TimerAlarm {
        TimerAlarm {
            registers: INSTANCES[instance],
            client: OptionalCell::empty(),
        }
    }

    fn clear_alarm(&self) {
        self.registers.events_compare[ALARM_COMPARE].write(Event::READY::CLEAR);
        self.registers.tasks_stop.write(Task::ENABLE::SET);
        self.registers.tasks_clear.write(Task::ENABLE::SET);
        self.disable_interrupts();
    }

    pub fn set_client(&self, client: &'static hil::time::Client) {
        self.client.set(client);
    }

    pub fn handle_interrupt(&self) {
        self.clear_alarm();
        self.client.map(|client| {
            client.fired();
        });
    }

    // Enable and disable interrupts use the bottom 4 bits
    // for the 4 compare interrupts. These functions shift
    // those bits to the correct place in the register.
    fn enable_interrupts(&self) {
        self.registers.intenset.write(ALARM_INTERRUPT_BIT_SET);
    }

    fn disable_interrupts(&self) {
        self.registers.intenclr.write(ALARM_INTERRUPT_BIT_SET);
    }

    fn interrupts_enabled(&self) -> bool {
        self.registers.intenset.is_set(ALARM_INTERRUPT_BIT)
    }

    fn value(&self) -> u32 {
        self.registers.tasks_capture[ALARM_CAPTURE].write(Task::ENABLE::SET);
        self.registers.cc[ALARM_CAPTURE].get()
    }
}

impl hil::time::Time for TimerAlarm {
    type Frequency = hil::time::Freq16KHz;

    fn disable(&self) {
        self.disable_interrupts();
    }

    fn is_armed(&self) -> bool {
        self.interrupts_enabled()
    }
}

impl hil::time::Alarm for TimerAlarm {
    fn now(&self) -> u32 {
        self.value()
    }

    fn set_alarm(&self, tics: u32) {
        self.disable_interrupts();
        self.registers.bitmode.write(Bitmode::BITMODE::Bit32);
        self.registers.cc[ALARM_COMPARE].write(CC::CC.val(tics));
        self.registers.tasks_start.write(Task::ENABLE::SET);
        self.enable_interrupts();
    }

    fn get_alarm(&self) -> u32 {
        self.registers.cc[ALARM_COMPARE].read(CC::CC)
    }
}
