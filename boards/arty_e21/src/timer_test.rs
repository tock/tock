#![allow(dead_code)]

use kernel::debug;
use kernel::hil::time::{self, Alarm, Ticks};

pub struct TimerTest<'a, A: Alarm<'a>> {
    alarm: &'a A,
}

impl<'a, A: Alarm<'a>> TimerTest<'a, A> {
    pub const fn new(alarm: &'a A) -> TimerTest<'a, A> {
        TimerTest { alarm: alarm }
    }

    pub fn start(&self) {
        debug!("starting");
        let start = self.alarm.now();
        let exp = start.wrapping_add(A::Ticks::from(99999u32));
        self.alarm.set_alarm(start, exp);
    }
}

impl<'a, A: Alarm<'a>> time::AlarmClient for TimerTest<'a, A> {
    fn alarm(&self) {
        debug!("timer!!");
    }
}
