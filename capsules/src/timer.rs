//! Provides userspace applications with a timer API.

use core::cell::Cell;
use kernel::{AppId, Callback, Grant, Driver, ReturnCode};
use kernel::hil::time::{self, Alarm, Frequency};
use kernel::process::Error;

/// Syscall driver number.
pub const DRIVER_NUM: usize = 0x00000000;

#[derive(Copy, Clone)]
enum Expiration {
    Disabled,
    Abs(u32),
}

#[derive(Copy, Clone)]
pub struct TimerData {
    t0: u32,
    expiration: Expiration,
    callback: Option<Callback>,
}

impl Default for TimerData {
    fn default() -> TimerData {
        TimerData {
            t0: 0,
            expiration: Expiration::Disabled,
            callback: None,
        }
    }
}

pub struct TimerDriver<'a, A: Alarm + 'a> {
    alarm: &'a A,
    num_armed: Cell<usize>,
    app_timer: Grant<TimerData>,
}

impl<'a, A: Alarm> TimerDriver<'a, A> {
    pub const fn new(alarm: &'a A, grant: Grant<TimerData>) -> TimerDriver<'a, A> {
        TimerDriver {
            alarm: alarm,
            num_armed: Cell::new(0),
            app_timer: grant,
        }
    }

    fn reset_active_timer(&self) {
        let now = self.alarm.now();
        let mut next_alarm = u32::max_value();
        let mut next_dist = u32::max_value();
        for timer in self.app_timer.iter() {
            timer.enter(|timer, _| match timer.expiration {
                Expiration::Abs(exp) => {
                    let t_dist = exp.wrapping_sub(now);
                    if next_dist > t_dist {
                        next_alarm = exp;
                        next_dist = t_dist;
                    }
                }
                Expiration::Disabled => {}
            });
        }
        if next_alarm != u32::max_value() {
            self.alarm.set_alarm(next_alarm);
        }
    }
}

impl<'a, A: Alarm> Driver for TimerDriver<'a, A> {
    /// Subscribe to timer expiration
    ///
    /// ### `_subscribe_num`
    ///
    /// - `0`: Subscribe to timer expiration
    fn subscribe(&self, _subscribe_num: usize, callback: Callback) -> ReturnCode {
        self.app_timer
            .enter(callback.app_id(), |td, _allocator| {
                td.callback = Some(callback);
                ReturnCode::SUCCESS
            })
            .unwrap_or_else(|err| err.into())
    }

    /// Setup and read the alarm.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Driver check.
    /// - `1`: Return the clock frequency in Hz.
    /// - `2`: Read the the current clock value
    /// - `3`: Stop the timer if it is outstanding
    /// - `4`: Set an alarm to fire at a given clock value `time`.
    fn command(&self, cmd_type: usize, data: usize, caller_id: AppId) -> ReturnCode {
        // Returns the error code to return to the user and whether we need to
        // reset which is the next active alarm. We only _don't_ reset if we're
        // disabling the underlying alarm anyway, if the underlying alarm is
        // currently disabled and we're enabling the first alarm, or on an error
        // (i.e. no change to the alarms).
        let (return_code, reset) = self.app_timer
            .enter(caller_id, |td, _alloc| {
                match cmd_type {
                    0 /* check if present */ => (ReturnCode::SuccessWithValue { value: 1 }, false),
                    1 /* Get clock frequency */ => {
                        let freq = <A::Frequency>::frequency() as usize;
                        (ReturnCode::SuccessWithValue { value: freq }, true)
                    },
                    2 /* capture time */ => {
                        let curr_time: u32 = self.alarm.now();
                        (ReturnCode::SuccessWithValue { value: curr_time as usize },
                         false)
                    },
                    3 /* Stop */ => {
                        let timer_id = data as u32;
                        match td.expiration {
                            Expiration::Disabled => {
                                // Request to stop when already stopped
                                (ReturnCode::EALREADY, false)
                            },
                            Expiration::Abs(exp) if exp != timer_id => {
                                // Request to stop invalid alarm id
                                (ReturnCode::EINVAL, false)
                            },
                            _ => {
                                td.expiration = Expiration::Disabled;
                                td.t0 = 0;
                                let new_num_armed = self.num_armed.get() - 1;
                                self.num_armed.set(new_num_armed);
                                if new_num_armed == 0 {
                                    self.alarm.disable();
                                    (ReturnCode::SUCCESS, false)
                                } else {
                                    (ReturnCode::SUCCESS, true)
                                }
                            }
                        }
                    },
                    4 /* Set absolute expiration */ => {
                        let time = data;
                        // if previously unarmed, but now will become armed
                        if let Expiration::Disabled = td.expiration {
                            self.num_armed.set(self.num_armed.get() + 1);
                        }


                        let now = self.alarm.now();
                        td.t0 = now;
                        td.expiration = Expiration::Abs(time as u32);

                        if self.alarm.is_armed() {
                            (ReturnCode::SuccessWithValue { value: time }, true)
                        } else {
                            self.alarm.set_alarm(time as u32);
                            (ReturnCode::SuccessWithValue { value: time}, false)
                        }
                    },
                    _ => (ReturnCode::ENOSUPPORT, false)
                }
            })
            .unwrap_or_else(|err| {
                let e = match err {
                    Error::OutOfMemory => ReturnCode::ENOMEM,
                    Error::AddressOutOfBounds => ReturnCode::EINVAL,
                    Error::NoSuchApp => ReturnCode::EINVAL,
                };
                (e, false)
            });
        if reset {
            self.reset_active_timer();
        }
        return_code
    }
}

impl<'a, A: Alarm> time::Client for TimerDriver<'a, A> {
    fn fired(&self) {
        let now = self.alarm.now();

        self.app_timer.each(|timer| if let Expiration::Abs(exp) = timer.expiration {
            let expired = now.wrapping_sub(timer.t0) >= exp.wrapping_sub(timer.t0);
            if expired {
                timer.expiration = Expiration::Disabled;
                self.num_armed.set(self.num_armed.get() - 1);
                timer.callback.map(|mut cb| cb.schedule(now as usize, exp as usize, 0));
            }
        });

        // If there are armed timers left, reset the underlying timer to the
        // nearest interval.  Otherwise, disable the underlying timer.
        if self.num_armed.get() > 0 {
            self.reset_active_timer();
        } else {
            self.alarm.disable();
        }
    }
}
