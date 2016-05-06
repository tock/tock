use core::cell::Cell;
use process::{AppId, Container, Callback};
use hil::Driver;
use hil::alarm::{Alarm, AlarmClient, Frequency};
use hil::timer::{Timer};

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

pub struct TimerDriver<'a, A: Alarm + 'a> {
    alarm: &'a A,
    num_armed: Cell<usize>,
    app_timer: Container<TimerData>
}

impl<'a, A: Alarm + 'a> TimerDriver<'a, A> {
    pub const fn new(alarm: &'a A, container: Container<TimerData>)
            -> TimerDriver<'a, A> {
        TimerDriver {
            alarm: alarm,
            num_armed: Cell::new(0),
            app_timer: container
        }
    }

    fn reset_active_timer(&self) {
        let now = self.alarm.now();
        let mut next_alarm = u32::max_value();
        let mut next_dist = u32::max_value();
        for timer in self.app_timer.iter() {
            timer.enter(|timer, _| {
                if timer.interval > 0 {
                    let native_int =
                        timer.interval * <A::Frequency>::frequency() / 1000;
                    let t_alarm = timer.t0.wrapping_add(native_int);
                    let t_dist = t_alarm.wrapping_sub(now);
                    if next_dist > t_dist {
                        next_alarm = t_alarm;
                        next_dist = t_dist;
                    }
                }
            });
        }
        if next_alarm != u32::max_value() {
            self.alarm.set_alarm(next_alarm);
        }
    }
}

impl<'a, A: Alarm> Driver for TimerDriver<'a, A> {
    fn subscribe(&self, _: usize, callback: Callback) -> isize {
        self.app_timer.enter(callback.app_id(), |td, _allocator| {
            td.callback = Some(callback);
            0
        }).unwrap_or(-1)
    }

    fn command(&self, cmd_type: usize, interval: usize, caller_id: AppId)
            -> isize {
        let interval = interval as u32;
        let (res, reset) = self.app_timer.enter(caller_id, |td, _allocator| {
            match cmd_type {
                2 /* Stop */ => {
                    if td.interval > 0 {
                        td.interval = 0;
                        td.t0 = 0;
                        let num_armed = self.num_armed.get();
                        self.num_armed.set(num_armed - 1);
                        if num_armed == 1 {
                            self.alarm.disable_alarm();
                            (0, false)
                        } else {
                            (0, true)
                        }
                    } else {
                        (-2, false)
                    }
                },
                /* 0 for Oneshot, 1 for Repeat */
                cmd_type if cmd_type <= 1 => {
                    if interval == 0 {
                        return (-2, false);
                    }

                    if td.interval == 0 {
                        self.num_armed.set(self.num_armed.get() + 1);
                    }

                    td.t0 = self.alarm.now();
                    td.interval = interval;

                    // Repeat if cmd_type was 1
                    td.repeating = cmd_type == 1;
                    if self.alarm.is_armed() {
                        (0, true)
                    } else {
                        let interval =
                            interval * <A::Frequency>::frequency() / 1000;
                        self.alarm.set_alarm(td.t0.wrapping_add(interval));
                        (0, false)
                    }
                },
                _ => (-1, false)
            }
        }).unwrap_or((-3, false));
        if reset {
            self.reset_active_timer();        
        }
        res
    }
}

impl<'a, A: Alarm> AlarmClient for TimerDriver<'a, A> {
    fn fired(&self) {
        let now = self.alarm.now();

        self.app_timer.each(|timer| {
            let elapsed = now.wrapping_sub(timer.t0);
            if timer.interval > 0 && elapsed >= timer.interval {
                if timer.repeating {
                    timer.t0 = now;
                } else {
                    timer.interval = 0;
                    self.num_armed.set(self.num_armed.get() - 1);
                }
                timer.callback.map(|mut cb| {
                    cb.schedule(now as usize, 0, 0);
                });
            }
        });
        if self.num_armed.get() > 0 {
            self.reset_active_timer();
        } else {
            self.alarm.disable_alarm();
        }
    }
}

