use core::cell::Cell;
use hil::alarm::{Alarm, AlarmClient};
use hil::timer::{Timer, TimerClient};

#[derive(Copy, Clone)]
enum Schedule {
    Oneshot,
    Repeating { interval: u32 }
}

pub struct AlarmToTimer<'a, Alrm: Alarm + 'a> {
    schedule: Cell<Schedule>,
    alarm: &'a Alrm,
    client: Cell<Option<&'a TimerClient>>
}

impl<'a, Alrm: Alarm> AlarmToTimer<'a, Alrm> {
    pub const fn new(alarm: &'a Alrm) -> AlarmToTimer<'a, Alrm> {
        AlarmToTimer {
            schedule: Cell::new(Schedule::Oneshot),
            alarm: alarm,
            client: Cell::new(None)
        }
    }

    pub fn set_client(&self, client: &'a TimerClient) {
        self.client.set(Some(client));
    }
}

impl<'a, Alrm: Alarm> Timer for AlarmToTimer<'a, Alrm> {
    fn now(&self) -> u32 {
        self.alarm.now()
    }

    fn oneshot(&self, interval: u32) {
        self.schedule.set(Schedule::Oneshot);

        let when = interval.wrapping_add(self.alarm.now());
        self.alarm.set_alarm(when);
    }

    fn repeat(&self, interval: u32) {
        self.schedule.set(Schedule::Repeating {interval: interval});

        let when = interval.wrapping_add(self.alarm.now());
        self.alarm.set_alarm(when);
    }
}

impl<'a, Alrm: Alarm> AlarmClient for AlarmToTimer<'a, Alrm> {
    fn fired(&self) {
        let now = self.now();

        match self.schedule.get() {
            Schedule::Oneshot => self.alarm.disable_alarm(),

            Schedule::Repeating { interval } => {
                let when = interval.wrapping_add(now);
                self.alarm.set_alarm(when);
            }
        }

        self.client.get().map(|client| client.fired(now) );
    }
}

