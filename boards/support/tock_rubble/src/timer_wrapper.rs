//! Mapping code for Rubble timer <-> Tock timer.

use kernel::hil::time;
use kernel::hil::time::Ticks;

pub struct TimerWrapper<'a, A>
where
    A: time::Alarm<'a>,
{
    alarm: &'a A,
}

impl<'a, A> TimerWrapper<'a, A>
where
    A: time::Alarm<'a>,
{
    pub fn new(alarm: &'a A) -> Self {
        TimerWrapper { alarm }
    }
}

impl<'a, A> rubble::time::Timer for TimerWrapper<'a, A>
where
    A: time::Alarm<'a>,
{
    fn now(&self) -> rubble::time::Instant {
        rubble::time::Instant::from_raw_micros(
            kernel::hil::rubble::types::Instant::from_alarm_time::<A>(self.alarm.now().into_u32())
                .raw_micros(),
        )
    }
}
