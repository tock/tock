//! Test that an Alarm implementation is working by trying a few edge
//! cases on the delay, including delays of 1 and 0 delays. Depends
//! on a working UART and debug! macro.
//!
//! Author: Philip Levis <plevis@google.com>
//! Last Modified: 6/17/2020
use core::cell::Cell;
use kernel::debug;
use kernel::hil::time::{Alarm, AlarmClient, Ticks};

pub struct TestAlarmEdgeCases<'a, A: 'a> {
    alarm: &'a A,
    counter: Cell<usize>,
    alarms: [u32; 20],
}

impl<'a, A: Alarm<'a>> TestAlarmEdgeCases<'a, A> {
    pub fn new(alarm: &'a A) -> TestAlarmEdgeCases<'a, A> {
        TestAlarmEdgeCases {
            alarm: alarm,
            counter: Cell::new(0),
            alarms:  [100,
                      200,
                      25, 25, 25, 25,
                      500,
                      0,
                      448,
                      15,
                      19,
                      1, 0, 33, 5,
                      1000,
                      27,
                      1,
                      0,
                      1],
        }
    }

    pub fn run(&self) {
        debug!("Starting alarm edge case tests.");
        self.set_next_alarm();
    }

    fn set_next_alarm(&self) {
        let counter = self.counter.get();
        let delay = A::ticks_from_ms(self.alarms[counter % 20]);
        let now = self.alarm.now();
        let start = now.wrapping_sub(A::Ticks::from(10));
        
        debug!("{}: Setting alarm to {} + {} = {}", now.into_u32(), start.into_u32(), delay.into_u32(), start.wrapping_add(delay).into_u32());
        self.alarm.set_alarm(start, delay);
        self.counter.set(counter + 1);
    }
}

impl<'a, A: Alarm<'a>> AlarmClient for TestAlarmEdgeCases<'a, A> {
    fn alarm(&self) {
        let now  = self.alarm.now();
        debug!("Alarm fired at {}.", now.into_u32());
        self.set_next_alarm();
    }

}
