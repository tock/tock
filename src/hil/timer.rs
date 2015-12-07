use core::cell::Cell;
use alarm::{Alarm, AlarmClient};

pub trait Timer {
    fn now(&self) -> u32;
    //fn cancel(&mut self, request: TimerRequest);
    fn oneshot(&self, interval: u32);
    fn repeat(&self, interval: u32);
}

pub trait TimerClient {
    fn fired(&self, now: u32);
}

pub struct SingleTimer<'a, Alrm: Alarm + 'a> {
    interval: Cell<u32>,
    when: Cell<u32>,
    repeat: Cell<bool>,
    alarm: &'a Alrm,
    client: Cell<Option<&'a TimerClient>>
}

unsafe impl<'a, A: Alarm + 'a> Sync for SingleTimer<'a, A> {}

impl<'a, Alrm: Alarm> SingleTimer<'a, Alrm> {
    pub const fn new(alarm: &'a Alrm) -> SingleTimer<'a, Alrm> {
        SingleTimer {
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

impl<'a, Alrm: Alarm> Timer for SingleTimer<'a, Alrm> {
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

impl<'a, Alrm: Alarm> AlarmClient for SingleTimer<'a, Alrm> {
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

