use core::cell::Cell;
use process::{AppId, Container, Callback};
use hil::Driver;
use hil::alarm::{Alarm, AlarmClient, Frequency};
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

    fn oneshot(&self, interval_ms: u32) {
        let interval = interval_ms * <Alrm::Frequency>::frequency() / 1000;

        self.schedule.set(Schedule::Oneshot);

        let when = interval.wrapping_add(self.alarm.now());
        self.alarm.set_alarm(when);
    }

    fn repeat(&self, interval_ms: u32) {
        let interval = interval_ms * <Alrm::Frequency>::frequency() / 1000;

        self.schedule.set(Schedule::Repeating {interval: interval});

        let when = interval.wrapping_add(self.alarm.now());
        self.alarm.set_alarm(when);
    }

    fn stop(&self) {
        self.alarm.disable_alarm();
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

#[derive(Copy, Clone)]
pub struct TimerData {
    t0: u32,
    interval: u32,
    repeating: bool,
    callback: Option<Callback>
}

impl Default for TimerData {
    fn default() -> TimerData {
        TimerData { t0: 0, interval: 0, repeating: false, callback: None }
    }
}

pub struct TimerDriver<'a, T: Timer + 'a> {
    timer: &'a T,
    app_timer: Container<TimerData>
}

impl<'a, T: Timer> TimerDriver<'a, T> {
    pub const fn new(timer: &'a T, container: Container<TimerData>)
            -> TimerDriver<'a, T> {
        TimerDriver {
            timer: timer,
            app_timer: container
        }
    }
}

impl<'a, T: Timer> Driver for TimerDriver<'a, T> {
    fn subscribe(&self, _: usize, callback: Callback) -> isize {
        self.app_timer.enter(callback.app_id(), |td, _allocator| {
            td.callback = Some(callback);
            0
        }).unwrap_or(-1)
    }

    fn command(&self, cmd_type: usize, interval: usize, caller_id: AppId)
            -> isize {
        let interval = interval as u32;
        self.app_timer.enter(caller_id, |td, _allocator| {
            match cmd_type {
                0 /* Oneshot */ => {
                    td.t0 = self.timer.now();
                    td.interval = interval;
                    td.repeating = false;
                    self.timer.oneshot(interval);
                    0
                },
                1 /* Repeating */ => {
                    td.t0 = self.timer.now();
                    td.interval = interval;
                    td.repeating = true;
                    self.timer.repeat(interval);
                    0
                },
                2 /* Stop */ => {
                    td.interval = 0;
                    td.t0 = 0;
                    self.timer.stop();
                    0
                },
                _ => -1
            }
        }).unwrap_or(-2)
    }
}

impl<'a, T: Timer> TimerClient for TimerDriver<'a, T> {
    fn fired(&self, now: u32) {
        self.app_timer.each(|timer| {
            let elapsed = now.wrapping_sub(timer.t0);
            if elapsed >= timer.interval {
                timer.callback.map(|mut cb| {
                    cb.schedule(now as usize, 0, 0);
                });
            }
        });
    }
}

