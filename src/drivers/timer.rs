use core::cell::Cell;
use hil::alarm::{Alarm, AlarmClient};
use hil::timer::{Timer, TimerClient};

pub struct AlarmToTimer<'a, Alrm: Alarm + 'a> {
    interval: Cell<u32>,
    when: Cell<u32>,
    repeat: Cell<bool>,
    alarm: &'a Alrm,
    client: Cell<Option<&'a TimerClient>>
}

impl<'a, Alrm: Alarm> AlarmToTimer<'a, Alrm> {
    pub const fn new(alarm: &'a Alrm) -> AlarmToTimer<'a, Alrm> {
        AlarmToTimer {
            interval: Cell::new(0),
            when: Cell::new(0),
            repeat: Cell::new(false),
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
        let when = interval.wrapping_add(self.alarm.now());

        self.when.set(when);

        self.interval.set(interval);
        self.repeat.set(false);

        self.alarm.set_alarm(when);
    }

    fn repeat(&self, interval: u32) {
        let when = interval.wrapping_add(self.alarm.now());

        self.when.set(when);

        self.interval.set(interval);
        self.repeat.set(true);

        self.alarm.set_alarm(when);
    }
}

impl<'a, Alrm: Alarm> AlarmClient for AlarmToTimer<'a, Alrm> {
    fn fired(&self) {
        let now = self.now();
        let repeat = self.repeat.get();
        if repeat {
            let interval = self.interval.get();
            let when = interval.wrapping_add(now);

            self.when.set(when);

            self.alarm.set_alarm(when);
        } else {
            self.alarm.disable_alarm();
        }
        self.client.get().map(|client| client.fired(now) );
    }
}

