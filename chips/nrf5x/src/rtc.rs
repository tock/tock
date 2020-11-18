//! RTC driver, nRF5X-family

use core::cell::Cell;
use kernel::common::cells::OptionalCell;
use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use kernel::hil::time::{self, Alarm, Ticks, Time};
use kernel::ReturnCode;

const RTC1_BASE: StaticRef<RtcRegisters> =
    unsafe { StaticRef::new(0x40011000 as *const RtcRegisters) };

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
    /// Compare event on CC\[n\] match.
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
    counter: ReadOnly<u32, Counter::Register>,
    /// 12-bit prescaler for COUNTER frequency (32768/(PRESCALER+1)).
    /// Must be written when RTC is stopped.
    prescaler: ReadWrite<u32, Prescaler::Register>,
    _reserved5: [u8; 52],
    /// Capture/compare registers.
    cc: [ReadWrite<u32, Counter::Register>; 4],
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
        /// Enable interrupt on COMPARE\[0\] event.
        COMPARE0 16,
        /// Enable interrupt on COMPARE\[1\] event.
        COMPARE1 17,
        /// Enable interrupt on COMPARE\[2\] event.
        COMPARE2 18,
        /// Enable interrupt on COMPARE\[3\] event.
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
    Counter [
        VALUE OFFSET(0) NUMBITS(24)
    ]
];

pub struct Rtc<'a> {
    registers: StaticRef<RtcRegisters>,
    overflow_client: OptionalCell<&'a dyn time::OverflowClient>,
    alarm_client: OptionalCell<&'a dyn time::AlarmClient>,
    enabled: Cell<bool>,
}

impl<'a> Rtc<'a> {
    pub const fn new() -> Self {
        Self {
            registers: RTC1_BASE,
            overflow_client: OptionalCell::empty(),
            alarm_client: OptionalCell::empty(),
            enabled: Cell::new(false),
        }
    }

    pub fn handle_interrupt(&self) {
        if self.registers.events_ovrflw.is_set(Event::READY) {
            self.registers.events_ovrflw.write(Event::READY::CLEAR);
            self.overflow_client.map(|client| client.overflow());
        }
        if self.registers.events_compare[0].is_set(Event::READY) {
            self.registers.intenclr.write(Inte::COMPARE0::SET);
            self.registers.events_compare[0].write(Event::READY::CLEAR);
            self.alarm_client.map(|client| {
                client.alarm();
            });
        }
    }
}

impl Time for Rtc<'_> {
    type Frequency = time::Freq32KHz;
    type Ticks = time::Ticks24;

    fn now(&self) -> Self::Ticks {
        Self::Ticks::from(self.registers.counter.read(Counter::VALUE))
    }
}

impl<'a> time::Counter<'a> for Rtc<'a> {
    fn set_overflow_client(&'a self, client: &'a dyn time::OverflowClient) {
        self.overflow_client.set(client);
        self.registers.intenset.write(Inte::OVRFLW::SET);
    }

    fn start(&self) -> ReturnCode {
        self.registers.prescaler.write(Prescaler::PRESCALER.val(0));
        self.registers.tasks_start.write(Task::ENABLE::SET);
        self.enabled.set(true);
        ReturnCode::SUCCESS
    }

    fn stop(&self) -> ReturnCode {
        //self.registers.cc[0].write(Counter::VALUE.val(0));
        self.registers.tasks_stop.write(Task::ENABLE::SET);
        self.enabled.set(false);
        ReturnCode::SUCCESS
    }

    fn reset(&self) -> ReturnCode {
        self.registers.tasks_clear.write(Task::ENABLE::SET);
        ReturnCode::SUCCESS
    }

    fn is_running(&self) -> bool {
        self.enabled.get()
    }
}

impl<'a> Alarm<'a> for Rtc<'a> {
    fn set_alarm_client(&self, client: &'a dyn time::AlarmClient) {
        self.alarm_client.set(client);
    }

    fn set_alarm(&self, reference: Self::Ticks, dt: Self::Ticks) {
        const SYNC_TICS: u32 = 2;
        let regs = &*self.registers;

        let mut expire = reference.wrapping_add(dt);

        let now = self.now();
        let earliest_possible = now.wrapping_add(Self::Ticks::from(SYNC_TICS));

        if !now.within_range(reference, expire) || expire.wrapping_sub(now).into_u32() <= SYNC_TICS
        {
            expire = earliest_possible;
        }

        regs.cc[0].write(Counter::VALUE.val(expire.into_u32()));
        regs.events_compare[0].write(Event::READY::CLEAR);
        regs.intenset.write(Inte::COMPARE0::SET);
    }

    fn get_alarm(&self) -> Self::Ticks {
        Self::Ticks::from(self.registers.cc[0].read(Counter::VALUE))
    }

    fn disarm(&self) -> ReturnCode {
        let regs = &*self.registers;
        regs.intenclr.write(Inte::COMPARE0::SET);
        regs.events_compare[0].write(Event::READY::CLEAR);
        ReturnCode::SUCCESS
    }

    fn is_armed(&self) -> bool {
        self.registers.evten.is_set(Inte::COMPARE0)
    }

    fn minimum_dt(&self) -> Self::Ticks {
        // TODO: not tested, arbitrary value
        Self::Ticks::from(10)
    }
}
