//! Provides userspace applications with a alarm API.

use core::cell::Cell;
use kernel::{AppId, Callback, Driver, Grant, ReturnCode};
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
pub struct AlarmData {
    t0: u32,
    expiration: Expiration,
    callback: Option<Callback>,
}

impl Default for AlarmData {
    fn default() -> AlarmData {
        AlarmData {
            t0: 0,
            expiration: Expiration::Disabled,
            callback: None,
        }
    }
}

pub struct AlarmDriver<'a, A: Alarm + 'a> {
    alarm: &'a A,
    num_armed: Cell<usize>,
    app_alarm: Grant<AlarmData>,
}

impl<'a, A: Alarm> AlarmDriver<'a, A> {
    pub const fn new(alarm: &'a A, grant: Grant<AlarmData>) -> AlarmDriver<'a, A> {
        AlarmDriver {
            alarm: alarm,
            num_armed: Cell::new(0),
            app_alarm: grant,
        }
    }

    fn reset_active_alarm(&self) {
        let now = self.alarm.now();
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
        }
    }
}

impl<'a, A: Alarm> Driver for AlarmDriver<'a, A> {
    /// Subscribe to alarm expiration
    ///
    /// ### `_subscribe_num`
    ///
    /// - `0`: Subscribe to alarm expiration
    fn subscribe(&self, _subscribe_num: usize, callback: Callback) -> ReturnCode {
        self.app_alarm
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
    /// - `3`: Stop the alarm if it is outstanding
    /// - `4`: Set an alarm to fire at a given clock value `time`.
    fn command(&self, cmd_type: usize, data: usize, _: usize, caller_id: AppId) -> ReturnCode {
        // Returns the error code to return to the user and whether we need to
        // reset which is the next active alarm. We only _don't_ reset if we're
        // disabling the underlying alarm anyway, if the underlying alarm is
        // currently disabled and we're enabling the first alarm, or on an error
        // (i.e. no change to the alarms).
        let (return_code, reset) = self.app_alarm
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
            self.reset_active_alarm();
        }
        return_code
    }
}

impl<'a, A: Alarm> time::Client for AlarmDriver<'a, A> {
    fn fired(&self) {
        let now = self.alarm.now();

        self.app_alarm.each(|alarm| {
            if let Expiration::Abs(exp) = alarm.expiration {
                let expired = now.wrapping_sub(alarm.t0) >= exp.wrapping_sub(alarm.t0);
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
        if self.num_armed.get() > 0 {
            self.reset_active_alarm();
        } else {
            self.alarm.disable();
        }
    }
}
