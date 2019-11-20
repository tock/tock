//! Provides userspace applications with a alarm API.

use core::cell::Cell;
use kernel::hil::time::{self, Alarm, Frequency};
use kernel::{AppId, Callback, Driver, Grant, ReturnCode};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Alarm as usize;

#[derive(Copy, Clone, Debug)]
enum Expiration {
    Disabled,
    Abs(u32),
}

#[derive(Copy, Clone)]
pub struct AlarmData {
    expiration: Expiration,
    callback: Option<Callback>,
}

impl Default for AlarmData {
    fn default() -> AlarmData {
        AlarmData {
            expiration: Expiration::Disabled,
            callback: None,
        }
    }
}

pub struct AlarmDriver<'a, A: Alarm<'a>> {
    alarm: &'a A,
    num_armed: Cell<usize>,
    app_alarm: Grant<AlarmData>,
    prev: Cell<u32>,
}

impl<A: Alarm<'a>> AlarmDriver<'a, A> {
    pub const fn new(alarm: &'a A, grant: Grant<AlarmData>) -> AlarmDriver<'a, A> {
        AlarmDriver {
            alarm: alarm,
            num_armed: Cell::new(0),
            app_alarm: grant,
            prev: Cell::new(0),
        }
    }

    fn reset_active_alarm(&self, now: u32) -> Option<u32> {
        self.prev.set(now);
        let mut next_alarm = u32::max_value();
        let mut next_dist = u32::max_value();
        for alarm in self.app_alarm.iter() {
            alarm.enter(|alarm, _| match alarm.expiration {
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
            Some(next_alarm)
        } else {
            None
        }
    }
}

impl<A: Alarm<'a>> Driver for AlarmDriver<'a, A> {
    /// Subscribe to alarm expiration
    ///
    /// ### `_subscribe_num`
    ///
    /// - `0`: Subscribe to alarm expiration
    fn subscribe(
        &self,
        _subscribe_num: usize,
        callback: Option<Callback>,
        app_id: AppId,
    ) -> ReturnCode {
        self.app_alarm
            .enter(app_id, |td, _allocator| {
                td.callback = callback;
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
    /// - `3`: Stop the alarm if it is outstanding
    /// - `4`: Set an alarm to fire at a given clock value `time`.
    fn command(&self, cmd_type: usize, data: usize, _: usize, caller_id: AppId) -> ReturnCode {
        // Returns the error code to return to the user and whether we need to
        // reset which is the next active alarm. We only _don't_ reset if we're
        // disabling the underlying alarm anyway, if the underlying alarm is
        // currently disabled and we're enabling the first alarm, or on an error
        // (i.e. no change to the alarms).
        self.app_alarm
            .enter(caller_id, |td, _alloc| {
                let now = self.alarm.now();
                let (return_code, reset) = match cmd_type {
                    0 /* check if present */ => (ReturnCode::SuccessWithValue { value: 1 }, false),
                    1 /* Get clock frequency */ => {
                        let freq = <A::Frequency>::frequency() as usize;
                        (ReturnCode::SuccessWithValue { value: freq }, false)
                    },
                    2 /* capture time */ => {
                        (ReturnCode::SuccessWithValue { value: now as usize },
                         false)
                    },
                    3 /* Stop */ => {
                        let alarm_id = data as u32;
                        match td.expiration {
                            Expiration::Disabled => {
                                // Request to stop when already stopped
                                (ReturnCode::EALREADY, false)
                            },
                            Expiration::Abs(exp) if exp != alarm_id => {
                                // Request to stop invalid alarm id
                                (ReturnCode::EINVAL, false)
                            },
                            _ => {
                                td.expiration = Expiration::Disabled;
                                let new_num_armed = self.num_armed.get() - 1;
                                self.num_armed.set(new_num_armed);
                                (ReturnCode::SUCCESS, true)
                            }
                        }
                    },
                    4 /* Set absolute expiration */ => {
                        let time = data;
                        // if previously unarmed, but now will become armed
                        if let Expiration::Disabled = td.expiration {
                            self.num_armed.set(self.num_armed.get() + 1);
                        }
                        td.expiration = Expiration::Abs(time as u32);
                        (ReturnCode::SuccessWithValue { value: time }, true)
                    },
                    _ => (ReturnCode::ENOSUPPORT, false)
                };
                if reset {
                    self.reset_active_alarm(now);
                }
                return_code
            })
            .unwrap_or_else(|err| err.into())
    }
}

fn has_expired(alarm: u32, now: u32, prev: u32) -> bool {
    now.wrapping_sub(prev) >= alarm.wrapping_sub(prev)
}

impl<A: Alarm<'a>> time::AlarmClient for AlarmDriver<'a, A> {
    fn fired(&self) {
        let now = self.alarm.now();
        self.app_alarm.each(|alarm| {
            if let Expiration::Abs(exp) = alarm.expiration {
                let expired = has_expired(exp, now, self.prev.get());
                if expired {
                    alarm.expiration = Expiration::Disabled;
                    self.num_armed.set(self.num_armed.get() - 1);
                    alarm
                        .callback
                        .map(|mut cb| cb.schedule(now as usize, exp as usize, 0));
                }
            }
        });

        // If there are armed alarms left, reset the underlying alarm to the
        // nearest interval.  Otherwise, disable the underlying alarm.
        if self.num_armed.get() == 0 {
            self.alarm.disable();
        } else if let Some(next_alarm) = self.reset_active_alarm(now) {
            let new_now = self.alarm.now();
            if has_expired(next_alarm, new_now, now) {
                self.fired();
            }
        } else {
            self.alarm.disable();
        }
    }
}
