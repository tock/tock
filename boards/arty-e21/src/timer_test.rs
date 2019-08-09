#![allow(dead_code)]

use kernel::debug;
use kernel::hil::time::{self, Alarm};

pub struct TimerTest<'a, A: Alarm> {
    alarm: &'a A,
}

impl<A: Alarm> TimerTest<'a, A> {
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

impl<A: Alarm> time::Client for TimerTest<'a, A> {
    fn fired(&self) {
        debug!("timer!!");
    }
}
