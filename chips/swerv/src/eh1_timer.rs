//! Internal Timer

use kernel::hil::time;
use kernel::hil::time::{Alarm, Counter, Ticks, Ticks32, Time};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::register_bitfields;
use kernel::ErrorCode;
use riscv_csr::csr::ReadWriteRiscvCsr;

/// 50MHz `Frequency`
#[derive(Debug)]
pub struct Freq50MHz;
impl time::Frequency for Freq50MHz {
    fn frequency() -> u32 {
        50_000_000
    }
}

pub enum TimerNumber {
    ZERO,
    ONE,
}

register_bitfields![usize,
    MITCNT [
        COUNT OFFSET(0) NUMBITS(32) []
    ],
    MITB [
        BOUND OFFSET(0) NUMBITS(32) []
    ],
    MITCTL [
        ENABLE OFFSET(0) NUMBITS(1) [],
        HALT_EN OFFSET(1) NUMBITS(1) [],
        PAUSE_EN OFFSET(2) NUMBITS(1) [],
    ],
];

pub struct Timer<'a> {
    number: TimerNumber,
    alarm_client: OptionalCell<&'a dyn time::AlarmClient>,

    mitcnt0: ReadWriteRiscvCsr<usize, MITCNT::Register, 0x7D2>,
    mitcnt1: ReadWriteRiscvCsr<usize, MITCNT::Register, 0x7D5>,

    mitb0: ReadWriteRiscvCsr<usize, MITB::Register, 0x7D3>,
    mitb1: ReadWriteRiscvCsr<usize, MITB::Register, 0x7D6>,

    mitctl0: ReadWriteRiscvCsr<usize, MITCTL::Register, 0x7D4>,
    mitctl1: ReadWriteRiscvCsr<usize, MITCTL::Register, 0x7D7>,
}

impl Timer<'_> {
    pub const fn new(number: TimerNumber) -> Self {
        Timer {
            number,
            alarm_client: OptionalCell::empty(),
            mitcnt0: ReadWriteRiscvCsr::new(),
            mitcnt1: ReadWriteRiscvCsr::new(),
            mitb0: ReadWriteRiscvCsr::new(),
            mitb1: ReadWriteRiscvCsr::new(),
            mitctl0: ReadWriteRiscvCsr::new(),
            mitctl1: ReadWriteRiscvCsr::new(),
        }
    }

    pub fn handle_interrupt(&self) {
        let _ = self.stop();
        self.alarm_client.map(|client| {
            client.alarm();
        });
    }
}

impl time::Time for Timer<'_> {
    type Frequency = Freq50MHz;
    type Ticks = Ticks32;

    fn now(&self) -> Self::Ticks {
        match self.number {
            TimerNumber::ZERO => Self::Ticks::from(self.mitcnt0.get() as u32),
            TimerNumber::ONE => Self::Ticks::from(self.mitcnt1.get() as u32),
        }
    }
}

impl<'a> Counter<'a> for Timer<'a> {
    fn set_overflow_client(&'a self, _client: &'a dyn time::OverflowClient) {
        // We have no way to know when this happens
    }

    fn start(&self) -> Result<(), ErrorCode> {
        match self.number {
            TimerNumber::ZERO => self.mitctl0.modify(MITCTL::ENABLE::SET),
            TimerNumber::ONE => self.mitctl1.modify(MITCTL::ENABLE::SET),
        };

        Ok(())
    }

    fn stop(&self) -> Result<(), ErrorCode> {
        match self.number {
            TimerNumber::ZERO => self.mitctl0.modify(MITCTL::ENABLE::CLEAR),
            TimerNumber::ONE => self.mitctl1.modify(MITCTL::ENABLE::CLEAR),
        };

        Ok(())
    }

    fn reset(&self) -> Result<(), ErrorCode> {
        // A counter is only cleared when it is equal or greater then
        // mitb.
        Err(ErrorCode::FAIL)
    }

    fn is_running(&self) -> bool {
        match self.number {
            TimerNumber::ZERO => self.mitctl0.is_set(MITCTL::ENABLE),
            TimerNumber::ONE => self.mitctl1.is_set(MITCTL::ENABLE),
        }
    }
}

impl<'a> Alarm<'a> for Timer<'a> {
    fn set_alarm_client(&self, client: &'a dyn time::AlarmClient) {
        self.alarm_client.set(client);
    }

    fn set_alarm(&self, reference: Self::Ticks, dt: Self::Ticks) {
        // Start the counter
        if !self.is_running() {
            let _ = Counter::start(self);
        }

        let now = self.now();
        let mut expire = reference.wrapping_add(dt);

        if !now.within_range(reference, expire) {
            expire = now;
        }

        let mut val = expire.into_usize();

        // 0xFFFF_FFFF is reserved to indicate disabled, don't set that value
        if val == 0xFFFF_FFFF {
            val += 1;
        }

        match self.number {
            TimerNumber::ZERO => self.mitb0.write(MITB::BOUND.val(val)),
            TimerNumber::ONE => self.mitb1.write(MITB::BOUND.val(val)),
        }
    }

    fn get_alarm(&self) -> Self::Ticks {
        match self.number {
            TimerNumber::ZERO => Self::Ticks::from(self.mitb0.read(MITB::BOUND) as u32),
            TimerNumber::ONE => Self::Ticks::from(self.mitb1.read(MITB::BOUND) as u32),
        }
    }

    fn disarm(&self) -> Result<(), ErrorCode> {
        match self.number {
            TimerNumber::ZERO => self.mitb0.write(MITB::BOUND.val(0xFFFF_FFFF)),
            TimerNumber::ONE => self.mitb1.write(MITB::BOUND.val(0xFFFF_FFFF)),
        };

        Ok(())
    }

    fn is_armed(&self) -> bool {
        match self.number {
            TimerNumber::ZERO => self.mitb0.read(MITB::BOUND) != 0xFFFF_FFFF,
            TimerNumber::ONE => self.mitb1.read(MITB::BOUND) != 0xFFFF_FFFF,
        }
    }

    fn minimum_dt(&self) -> Self::Ticks {
        Self::Ticks::from(1 as u32)
    }
}

impl kernel::platform::scheduler_timer::SchedulerTimer for Timer<'_> {
    fn start(&self, us: u32) {
        let now = self.now();
        let tics = Self::ticks_from_us(us);
        self.set_alarm(now, tics);
    }

    fn reset(&self) {
        let _ = self.stop();
    }

    fn arm(&self) {
        // start() has already armed the timer
    }

    fn disarm(&self) {
        // We have no way to "disarm" a timer with stopping it unless we
        // mask the interrupt in the PIC
    }

    fn get_remaining_us(&self) -> Option<u32> {
        let alarm = self.get_alarm();
        let now = self.now();

        if alarm > now {
            Some(alarm.wrapping_sub(now).into_u32())
        } else {
            None
        }
    }
}
