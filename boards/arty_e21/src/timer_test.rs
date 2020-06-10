#![allow(dead_code)]

use kernel::debug;
use kernel::hil::time::{self, Alarm};

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
        let exp = start + 99999;
        self.alarm.set_alarm(exp);
    }
}

impl<'a, A: Alarm<'a>> time::AlarmClient for TimerTest<'a, A> {
    fn fired(&self) {
        debug!("timer!!");
    }
}
