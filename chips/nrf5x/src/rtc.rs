//! RTC driver, nRF5X-family

use kernel::common::cells::OptionalCell;
use kernel::common::regs::{ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use kernel::hil::time::{self, Alarm, Freq32KHz, Time};
use kernel::hil::Controller;

const RTC1_BASE: *const RtcRegisters = 0x40011000 as *const RtcRegisters;

#[repr(C)]
struct RtcRegisters {
    /// Start RTC Counter.
    tasks_start: WriteOnly<u32, Task::Register>,
    /// Stop RTC Counter.
    tasks_stop: WriteOnly<u32, Task::Register>,
    /// Clear RTC Counter.
    tasks_clear: WriteOnly<u32, Task::Register>,
    /// Set COUNTER to 0xFFFFFFF0.
    tasks_trigovrflw: WriteOnly<u32, Task::Register>,
    _reserved0: [u8; 240],
    /// Event on COUNTER increment.
    events_tick: ReadWrite<u32, Event::Register>,
    /// Event on COUNTER overflow.
    events_ovrflw: ReadWrite<u32, Event::Register>,
    _reserved1: [u8; 56],
    /// Compare event on CC[n] match.
    events_compare: [ReadWrite<u32, Event::Register>; 4],
    _reserved2: [u8; 436],
    /// Interrupt enable set register.
    intenset: ReadWrite<u32, Inte::Register>,
    /// Interrupt enable clear register.
    intenclr: ReadWrite<u32, Inte::Register>,
    _reserved3: [u8; 52],
    /// Configures event enable routing to PPI for each RTC event.
    evten: ReadWrite<u32, Inte::Register>,
    /// Enable events routing to PPI.
    evtenset: ReadWrite<u32, Inte::Register>,
    /// Disable events routing to PPI.
    evtenclr: ReadWrite<u32, Inte::Register>,
    _reserved4: [u8; 440],
    /// Current COUNTER value.
    counter: ReadOnly<u32>,
    /// 12-bit prescaler for COUNTER frequency (32768/(PRESCALER+1)).
    /// Must be written when RTC is stopped.
    prescaler: ReadWrite<u32, Prescaler::Register>,
    _reserved5: [u8; 52],
    /// Capture/compare registers.
    cc: [ReadWrite<u32, CC::Register>; 4],
    _reserved6: [u8; 2732],
    /// Peripheral power control.
    power: ReadWrite<u32>,
}

register_bitfields![u32,
    Inte [
        /// Enable interrupt on TICK event.
        TICK 0,
        /// Enable interrupt on OVRFLW event.
        OVRFLW 1,
        /// Enable interrupt on COMPARE[0] event.
        COMPARE0 16,
        /// Enable interrupt on COMPARE[1] event.
        COMPARE1 17,
        /// Enable interrupt on COMPARE[2] event.
        COMPARE2 18,
        /// Enable interrupt on COMPARE[3] event.
        COMPARE3 19
    ],
    Prescaler [
        PRESCALER OFFSET(0) NUMBITS(12)
    ],
    Task [
        ENABLE 0
    ],
    Event [
        READY 0
    ],
    CC [
        CC OFFSET(0) NUMBITS(24)
    ]
];

pub struct Rtc {
    registers: StaticRef<RtcRegisters>,
    callback: OptionalCell<&'static time::Client>,
}

pub static mut RTC: Rtc = Rtc {
    registers: unsafe { StaticRef::new(RTC1_BASE) },
    callback: OptionalCell::empty(),
};

impl Controller for Rtc {
    type Config = &'static time::Client;

    fn configure(&self, client: &'static time::Client) {
        self.callback.set(client);

        // FIXME: what to do here?
        // self.start();
        // Set counter incrementing frequency to 16KHz
        // rtc1().prescaler.set(1);
    }
}

impl Rtc {
    pub fn start(&self) {
        // This function takes a nontrivial amount of time
        // So it should only be called during initialization, not each tick
        self.registers.prescaler.write(Prescaler::PRESCALER.val(0));
        self.registers.tasks_start.write(Task::ENABLE::SET);
    }

    pub fn stop(&self) {
        self.registers.cc[0].write(CC::CC.val(0));
        self.registers.tasks_stop.write(Task::ENABLE::SET);
    }

    fn is_running(&self) -> bool {
        self.registers.evten.is_set(Inte::COMPARE0)
    }

    pub fn handle_interrupt(&self) {
        self.registers.events_compare[0].write(Event::READY::CLEAR);
        self.registers.intenclr.write(Inte::COMPARE0::SET);
        self.callback.map(|cb| {
            cb.fired();
        });
    }

    pub fn set_client(&self, client: &'static time::Client) {
        self.callback.set(client);
    }
}

impl Time for Rtc {
    type Frequency = Freq32KHz;

    fn disable(&self) {
        self.registers.intenclr.write(Inte::COMPARE0::SET);
    }

    fn is_armed(&self) -> bool {
        self.is_running()
    }
}

impl Alarm for Rtc {
    fn now(&self) -> u32 {
        self.registers.counter.get()
    }

    fn set_alarm(&self, tics: u32) {
        // Similarly to the disable function, here we don't restart the timer
        // Instead, we just listen for it again
        self.registers.cc[0].write(CC::CC.val(tics));
        self.registers.intenset.write(Inte::COMPARE0::SET);
    }

    fn get_alarm(&self) -> u32 {
        self.registers.cc[0].read(CC::CC)
    }
}
