#![allow(dead_code)]

use kernel::debug;
use kernel::hil::time::{Alarm, AlarmClient};

pub struct TimerTest<'a, A: Alarm<'a>> {
    alarm: &'a A,
}

impl<A: Alarm<'a>> TimerTest<'a, A> {
    pub const fn new(alarm: &'a A) -> TimerTest<'a, A> {
        TimerTest { alarm: alarm }
    }

    pub fn start(&self) {
        debug!("starting");
        self.alarm.set_alarm_from_now(A::Ticks::from(99999));
    }
}

impl<A: Alarm<'a>> AlarmClient for TimerTest<'a, A> {
    fn fired(&self) {
        debug!("timer!!");
    }
}
