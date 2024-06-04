// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

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
