//! Provides userspace applications with a alarm API.

use core::cell::Cell;
use kernel::common::cells::OptionalCell;
use kernel::hil::time::{Alarm, AlarmClient, Frequency, Ticks};
use kernel::{AppId, Callback, Driver, Grant, ReturnCode};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Alarm as usize;

#[derive(Copy, Clone, Debug)]
enum Expiration<T: Ticks> {
    Disabled,
    Abs(T),
}

#[derive(Copy, Clone)]
pub struct AlarmData<T: Ticks> {
    expiration: Expiration<T>,
    callback: Option<Callback>,
}

impl<T: Ticks> Default for AlarmData<T> {
    fn default() -> AlarmData<T> {
        AlarmData {
            expiration: Expiration::Disabled,
            callback: None,
        }
    }
}

pub struct AlarmDriver<'a, A: Alarm<'a>> {
    alarm: &'a A,
    num_armed: Cell<usize>,
    app_alarm: Grant<AlarmData<A::Ticks>>,
    prev: OptionalCell<A::Ticks>,
}

impl<A: Alarm<'a>> AlarmDriver<'a, A> {
    pub const fn new(alarm: &'a A, grant: Grant<AlarmData<A::Ticks>>) -> AlarmDriver<'a, A> {
        AlarmDriver {
            alarm: alarm,
            num_armed: Cell::new(0),
            app_alarm: grant,
            prev: OptionalCell::empty(),
        }
    }

    fn reset_active_alarm(&self, now: A::Ticks) -> Option<A::Ticks> {
        self.prev.set(now);
        let mut next_alarm = None;
        for alarm in self.app_alarm.iter() {
            alarm.enter(|alarm, _| match alarm.expiration {
                Expiration::Abs(exp) => match next_alarm {
                    None => next_alarm = Some(exp),
                    Some(next) => {
                        if A::Ticks::expired(now, next, exp) {
                            next_alarm = Some(exp);
                        }
                    }
                },
                Expiration::Disabled => {}
            });
        }
        if let Some(next) = next_alarm {
            self.alarm.set_alarm(next);
        }
        next_alarm
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
                        (ReturnCode::SuccessWithValue { value: now.into_usize() },
                         false)
                    },
                    3 /* Stop */ => {
                        let alarm_id = data;
                        match td.expiration {
                            Expiration::Disabled => {
                                // Request to stop when already stopped
                                (ReturnCode::EALREADY, false)
                            },
                            Expiration::Abs(exp) if exp.into_usize() != alarm_id => {
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
                        td.expiration = Expiration::Abs(A::Ticks::from(time as u32));
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

impl<A: Alarm<'a>> AlarmClient for AlarmDriver<'a, A> {
    fn fired(&self) {
        let now = self.alarm.now();
        self.app_alarm.each(|alarm| {
            if let Expiration::Abs(exp) = alarm.expiration {
                let expired = A::Ticks::expired(self.prev.unwrap_or(A::Ticks::from(0)), now, exp);
                if expired {
                    alarm.expiration = Expiration::Disabled;
                    self.num_armed.set(self.num_armed.get() - 1);
                    alarm
                        .callback
                        .map(|mut cb| cb.schedule(now.into_usize(), exp.into_usize(), 0));
                }
            }
        });

        // If there are armed alarms left, reset the underlying alarm to the
        // nearest interval.  Otherwise, disable the underlying alarm.
        if self.num_armed.get() == 0 {
            self.alarm.disable();
        } else if let Some(next_alarm) = self.reset_active_alarm(now) {
            let new_now = self.alarm.now();
            if A::Ticks::expired(now, new_now, next_alarm) {
                self.fired();
            }
        } else {
            self.alarm.disable();
        }
    }
}
