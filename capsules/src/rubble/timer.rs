//! Mapping code for Rubble timer <-> Tock timer.
use core::convert::{TryFrom, TryInto};

use kernel::hil::time::{Frequency, Time};

pub struct RubbleTimer<'a, A>
where
    A: kernel::hil::time::Alarm<'a>,
{
    alarm: &'a A,
}

impl<'a, A> RubbleTimer<'a, A>
where
    A: kernel::hil::time::Alarm<'a>,
{
    pub fn new(alarm: &'a A) -> Self {
        RubbleTimer { alarm }
    }
}

impl<'a, A> rubble::time::Timer for RubbleTimer<'a, A>
where
    A: kernel::hil::time::Alarm<'a>,
{
    fn now(&self) -> rubble::time::Instant {
        alarm_time_to_rubble_instant::<A>(self.alarm.now())
    }
}

pub fn alarm_time_to_rubble_instant<A: Time>(raw: u32) -> rubble::time::Instant {
    // Frequency::frequency() returns NOW_UNIT / second, and we want
    // microseconds. `now / frequency` gives us seconds, so
    // `now * 1000_000 / frequency` is microseconds

    // multiply before dividing to be as accurate as possible, and use u64 to
    // overflow.
    rubble::time::Instant::from_raw_micros(
        ((raw as u64 * 1000_000u64) / A::Frequency::frequency() as u64)
            .try_into()
            .unwrap(),
    )
}

pub fn rubble_instant_to_alarm_time<A: Time>(alarm: &A, instant: rubble::time::Instant) -> u32 {
    // instant.raw_micros() is microseconds, and we want NOW_UNIT.
    // Frequency::frequency() returns NOW_UNIT / second, so `raw_micros * frequency` gives us
    // `NOW_UNIT * microseconds / seconds`. `microseconds = 1000_000 seconds`,
    // so `raw_micros * frequency / 1000_000` is NOW_UNIT.
    u32::try_from(instant.raw_micros() as u64 * A::Frequency::frequency() as u64 / 1000_000u64)
        .unwrap()
        % alarm.max_tics()
}

#[cfg(test)]
mod test {
    use super::*;
    use kernel::hil::time::Freq32KHz;

    struct VAlarm;
    impl Time for VAlarm {
        type Frequency = Freq32KHz;
        fn now(&self) -> u32 {
            panic!()
        }
        fn max_tics(&self) -> u32 {
            !0u32
        }
    }

    #[test]
    fn time_roundtrip() {
        for &start in &[0, 3120, 10000, 22500, 9514094] {
            let rubble = alarm_time_to_rubble_instant::<VAlarm>(start);
            let end = rubble_instant_to_alarm_time::<VAlarm>(&VAlarm, rubble);
            assert!((start as i32 - end as i32).abs() < 10);
        }
    }
}
