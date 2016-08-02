use core::cell::Cell;
use process::{AppId, Container, Callback};
use hil::Driver;
use hil::alarm::{Alarm, AlarmClient, Frequency};

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
                    let t_alarm = timer.t0.wrapping_add(timer.interval);
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
        // First, convert from milliseconds to native clock frequency
        let interval = (interval as u32) * <A::Frequency>::frequency() / 1000;

        // Returns the error code to return to the user (0 for success, negative
        // otherwise) and whether we need to reset which is the next active
        // alarm. We only _don't_ reset if we're disabling the underlying alarm
        // anyway, if the underlying alarm is currently disabled and we're
        // enabling the first alarm, or on an error (i.e. no change to the
        // alarms).
        let (err_code, reset) = self.app_timer.enter(caller_id, |td, _alloc| {
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

                    // if previously unarmed, but now will become armed
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
                        self.alarm.set_alarm(td.t0.wrapping_add(td.interval));
                        (0, false)
                    }
                },
                _ => (-1, false)
            }
        }).unwrap_or((-3, false));
        if reset {
            self.reset_active_timer();
        }
        err_code
    }
}

impl<'a, A: Alarm> AlarmClient for TimerDriver<'a, A> {
    fn fired(&self) {
        let now = self.alarm.now();

        self.app_timer.each(|timer| {
            let elapsed = now.wrapping_sub(timer.t0);

            // timer.interval == 0 means the timer is inactive
            if timer.interval > 0 &&
                    // Becuse of the calculations done for timer.interval when
                    // setting the timer, we might fire earlier than expected
                    // by some jitter.
                    elapsed >= timer.interval {

                if timer.repeating {
                    // Repeating timer, reset the reference time to now
                    timer.t0 = now;
                } else {
                    // Deactivate timer
                    timer.interval = 0;
                    self.num_armed.set(self.num_armed.get() - 1);
                }

                timer.callback.map(|mut cb| {
                    cb.schedule(now as usize, 0, 0);
                });
            }
        });

        // If there are armed timers left, reset the underlying timer to the
        // nearest interval. Otherwise, disable the underlying timer.
        if self.num_armed.get() > 0 {
            self.reset_active_timer();
        } else {
            self.alarm.disable_alarm();
        }
    }
}

