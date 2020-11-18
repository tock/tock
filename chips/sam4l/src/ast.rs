//! Implementation of a single hardware timer.
//!
//! - Author: Amit Levy <levya@cs.stanford.edu>
//! - Author: Philip Levis <pal@cs.stanford.edu>
//! - Date: September 17, 2020

use crate::pm::{self, PBDClock};
use kernel::common::cells::OptionalCell;
use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use kernel::hil::time::{self, Ticks};
use kernel::hil::Controller;
use kernel::ReturnCode;

/// Minimum number of clock tics to make sure ALARM0 register is synchronized
///
/// The datasheet has the following ominous language (Section 19.5.3.2):
///
/// > Because of synchronization, the transfer of the alarm value will not
/// > happen immediately. When changing/setting the alarm value, the user must
/// > make sure that the counter will not count the selected alarm value before
/// > the value is transferred to the register. In that case, the first alarm
/// > interrupt after the change will not be triggered.
///
/// In practice, we've observed that when the alarm is set for a counter value
/// less than or equal to four tics ahead of the current counter value, the
/// alarm interrupt doesn't fire. Thus, we simply round up to at least eight
/// tics. Seems safe enough and in practice has seemed to work.
const ALARM0_SYNC_TICS: u32 = 8;

#[repr(C)]
struct AstRegisters {
    cr: ReadWrite<u32, Control::Register>,
    cv: ReadWrite<u32, Value::Register>,
    sr: ReadOnly<u32, Status::Register>,
    scr: WriteOnly<u32, Interrupt::Register>,
    ier: WriteOnly<u32, Interrupt::Register>,
    idr: WriteOnly<u32, Interrupt::Register>,
    imr: ReadOnly<u32, Interrupt::Register>,
    wer: ReadWrite<u32, Event::Register>,
    // 0x20
    ar0: ReadWrite<u32, Value::Register>,
    ar1: ReadWrite<u32, Value::Register>,
    _reserved0: [u32; 2],
    pir0: ReadWrite<u32, PeriodicInterval::Register>,
    pir1: ReadWrite<u32, PeriodicInterval::Register>,
    _reserved1: [u32; 2],
    // 0x40
    clock: ReadWrite<u32, ClockControl::Register>,
    dtr: ReadWrite<u32, DigitalTuner::Register>,
    eve: WriteOnly<u32, Event::Register>,
    evd: WriteOnly<u32, Event::Register>,
    evm: ReadOnly<u32, Event::Register>,
    calv: ReadWrite<u32, Calendar::Register>, // we leave out parameter and version
}

register_bitfields![u32,
    Control [
        /// Prescalar Select
        PSEL OFFSET(16) NUMBITS(5) [],
        /// Clear on Alarm 1
        CA1  OFFSET(9) NUMBITS(1) [
            NoClearCounter = 0,
            ClearCounter = 1
        ],
        /// Clear on Alarm 0
        CA0  OFFSET(8) NUMBITS(1) [
            NoClearCounter = 0,
            ClearCounter = 1
        ],
        /// Calendar Mode
        CAL  OFFSET(2) NUMBITS(1) [
            CounterMode = 0,
            CalendarMode = 1
        ],
        /// Prescalar Clear
        PCLR OFFSET(1) NUMBITS(1) [],
        /// Enable
        EN   OFFSET(0) NUMBITS(1) [
            Disable = 0,
            Enable = 1
        ]
    ],

    Value [
        VALUE OFFSET(0) NUMBITS(32) []
    ],

    Status [
        /// Clock Ready
        CLKRDY 29,
        /// Clock Busy
        CLKBUSY 28,
        /// AST Ready
        READY 25,
        /// AST Busy
        BUSY 24,
        /// Periodic 0
        PER0 16,
        /// Alarm 0
        ALARM0 8,
        /// Overflow
        OVF 0
    ],

    Interrupt [
        /// Clock Ready
        CLKRDY 29,
        /// AST Ready
        READY 25,
        /// Periodic 0
        PER0 16,
        /// Alarm 0
        ALARM0 8,
        /// Overflow
        OVF 0
    ],

    Event [
        /// Periodic 0
        PER0 16,
        /// Alarm 0
        ALARM0 8,
        /// Overflow
        OVF 0
    ],

    PeriodicInterval [
        /// Interval Select
        INSEL OFFSET(0) NUMBITS(5) []
    ],

    ClockControl [
        /// Clock Source Selection
        CSSEL OFFSET(8) NUMBITS(3) [
            RCSYS = 0,
            OSC32 = 1,
            APBClock = 2,
            GCLK = 3,
            Clk1k = 4
        ],
        /// Clock Enable
        CEN   OFFSET(0) NUMBITS(1) [
            Disable = 0,
            Enable = 1
        ]
    ],

    DigitalTuner [
        VALUE OFFSET(8) NUMBITS(8) [],
        ADD   OFFSET(5) NUMBITS(1) [],
        EXP   OFFSET(0) NUMBITS(5) []
    ],

    Calendar [
        YEAR  OFFSET(26) NUMBITS(6) [],
        MONTH OFFSET(22) NUMBITS(4) [],
        DAY   OFFSET(17) NUMBITS(5) [],
        HOUR  OFFSET(12) NUMBITS(5) [],
        MIN   OFFSET( 6) NUMBITS(6) [],
        SEC   OFFSET( 0) NUMBITS(6) []
    ]
];

const AST_ADDRESS: StaticRef<AstRegisters> =
    unsafe { StaticRef::new(0x400F0800 as *const AstRegisters) };

pub struct Ast<'a> {
    registers: StaticRef<AstRegisters>,
    callback: OptionalCell<&'a dyn time::AlarmClient>,
}

impl<'a> Ast<'a> {
    pub const fn new() -> Self {
        Self {
            registers: AST_ADDRESS,
            callback: OptionalCell::empty(),
        }
    }
}

impl Controller for Ast<'_> {
    type Config = &'static dyn time::AlarmClient;

    fn configure(&self, client: Self::Config) {
        self.callback.set(client);

        pm::enable_clock(pm::Clock::PBD(PBDClock::AST));
        self.select_clock(Clock::ClockOsc32);
        self.disable();
        self.disable_alarm_irq();
        self.set_prescalar(0); // 32KHz / (2^(0 + 1)) = 16KHz
        self.enable_alarm_wake();

        self.clear_alarm();
    }
}

#[repr(usize)]
#[allow(dead_code)]
enum Clock {
    ClockRCSys = 0,
    ClockOsc32 = 1,
    ClockAPB = 2,
    ClockGclk2 = 3,
    Clock1K = 4,
}

impl<'a> Ast<'a> {
    fn clock_busy(&self) -> bool {
        self.registers.sr.is_set(Status::CLKBUSY)
    }

    fn busy(&self) -> bool {
        self.registers.sr.is_set(Status::BUSY)
    }

    /// Clears the alarm bit in the status register (indicating the alarm value
    /// has been reached).
    fn clear_alarm(&self) {
        while self.busy() {}
        self.registers.scr.write(Interrupt::ALARM0::SET);
        while self.busy() {}
    }

    // Configure the clock to use to drive the AST
    fn select_clock(&self, clock: Clock) {
        // Disable clock by setting first bit to zero
        while self.clock_busy() {}
        self.registers.clock.modify(ClockControl::CEN::CLEAR);
        while self.clock_busy() {}

        // Select clock
        self.registers
            .clock
            .write(ClockControl::CSSEL.val(clock as u32));
        while self.clock_busy() {}

        // Re-enable clock
        self.registers.clock.modify(ClockControl::CEN::SET);
        while self.clock_busy() {}
    }

    /// Enables the AST registers
    fn enable(&self) {
        while self.busy() {}
        self.registers.cr.modify(Control::EN::SET);
        while self.busy() {}
    }

    /// Disable the AST registers
    fn disable(&self) {
        while self.busy() {}
        self.registers.cr.modify(Control::EN::CLEAR);
        while self.busy() {}
    }

    fn is_enabled(&self) -> bool {
        let regs: &AstRegisters = &*self.registers;
        while self.busy() {}
        regs.cr.is_set(Control::EN)
    }

    /// Returns if an alarm is currently set
    fn is_alarm_enabled(&self) -> bool {
        while self.busy() {}
        self.registers.sr.is_set(Status::ALARM0)
    }

    fn set_prescalar(&self, val: u8) {
        while self.busy() {}
        self.registers.cr.modify(Control::PSEL.val(val as u32));
        while self.busy() {}
    }

    fn enable_alarm_irq(&self) {
        self.registers.ier.write(Interrupt::ALARM0::SET);
    }

    fn disable_alarm_irq(&self) {
        self.registers.idr.write(Interrupt::ALARM0::SET);
    }

    fn enable_alarm_wake(&self) {
        while self.busy() {}
        self.registers.wer.modify(Event::ALARM0::SET);
        while self.busy() {}
    }

    fn get_counter(&self) -> u32 {
        while self.busy() {}
        self.registers.cv.read(Value::VALUE)
    }

    fn set_counter(&self, val: u32) {
        let regs: &AstRegisters = &*self.registers;
        while self.busy() {}
        regs.cv.set(val);
    }

    pub fn handle_interrupt(&self) {
        self.clear_alarm();
        self.callback.map(|cb| {
            cb.alarm();
        });
    }
}

impl time::Time for Ast<'_> {
    type Frequency = time::Freq16KHz;
    type Ticks = time::Ticks32;

    fn now(&self) -> Self::Ticks {
        Self::Ticks::from(self.get_counter())
    }
}

impl<'a> time::Counter<'a> for Ast<'a> {
    fn set_overflow_client(&'a self, _client: &'a dyn time::OverflowClient) {}

    fn start(&self) -> ReturnCode {
        self.enable();
        ReturnCode::SUCCESS
    }

    fn stop(&self) -> ReturnCode {
        self.disable();
        ReturnCode::SUCCESS
    }

    fn reset(&self) -> ReturnCode {
        self.set_counter(0);
        ReturnCode::SUCCESS
    }

    fn is_running(&self) -> bool {
        self.is_enabled()
    }
}

impl<'a> time::Alarm<'a> for Ast<'a> {
    fn set_alarm_client(&self, client: &'a dyn time::AlarmClient) {
        self.callback.set(client);
    }

    fn set_alarm(&self, reference: Self::Ticks, dt: Self::Ticks) {
        let now = Self::Ticks::from(self.get_counter());
        let mut expire = reference.wrapping_add(dt);
        if !now.within_range(reference, expire) {
            // We have already passed when: just fire ASAP
            // Note this will also trigger the increment below
            expire = Self::Ticks::from(now);
        }

        // Firing is too close in the future, delay it a bit
        // to make sure we don't miss the tick
        if expire.wrapping_sub(now).into_u32() <= ALARM0_SYNC_TICS {
            expire = now.wrapping_add(Self::Ticks::from(ALARM0_SYNC_TICS));
        }

        // Clear any alarm event that may be pending before setting new alarm.
        self.clear_alarm();

        while self.busy() {}
        self.registers
            .ar0
            .write(Value::VALUE.val(expire.into_u32()));

        while self.busy() {}
        self.enable_alarm_irq();
        self.enable();
    }

    fn get_alarm(&self) -> Self::Ticks {
        while self.busy() {}
        Self::Ticks::from(self.registers.ar0.read(Value::VALUE))
    }

    fn disarm(&self) -> ReturnCode {
        // After disabling the IRQ and clearing the alarm bit in the
        // status register, the NVIC bit is also guaranteed to be clear.
        self.disable_alarm_irq();
        self.clear_alarm();
        ReturnCode::SUCCESS
    }

    fn is_armed(&self) -> bool {
        self.is_alarm_enabled()
    }

    fn minimum_dt(&self) -> Self::Ticks {
        Self::Ticks::from(ALARM0_SYNC_TICS)
    }
}
