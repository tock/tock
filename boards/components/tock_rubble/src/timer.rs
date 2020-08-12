//! Mapping code for Rubble timer <-> Tock timer.
use kernel::hil::rubble as rubble_hil;
use kernel::hil::time::Alarm;

pub struct TimerWrapper<'a, A>
where
    A: Alarm<'a>,
{
    alarm: &'a A,
}

impl<'a, A> TimerWrapper<'a, A>
where
    A: Alarm<'a>,
{
    pub fn new(alarm: &'a A) -> Self {
        TimerWrapper { alarm }
    }
}

impl<'a, A> rubble::time::Timer for TimerWrapper<'a, A>
where
    A: Alarm<'a>,
{
    fn now(&self) -> rubble::time::Instant {
        rubble::time::Instant::from_raw_micros(
            rubble_hil::Instant::from_alarm_time::<A>(self.alarm.now()).microseconds,
        )
    }
}
