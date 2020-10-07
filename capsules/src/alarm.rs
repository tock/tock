//! Tock syscall driver capsule for Alarms, which issue callbacks when
//! a point in time has been reached.

use core::cell::Cell;
use kernel::hil::time::{self, Alarm, Frequency, Ticks, Ticks32};
use kernel::{AppId, Callback, Driver, Grant, ReturnCode};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Alarm as usize;

#[derive(Copy, Clone, Debug)]
enum Expiration<T: Ticks> {
    Disabled,
    Enabled { reference: T, dt: T },
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
    app_alarms: Grant<AlarmData<A::Ticks>>,
    next_alarm: Cell<Expiration<A::Ticks>>,
}

impl<'a, A: Alarm<'a>> AlarmDriver<'a, A> {
    pub const fn new(alarm: &'a A, grant: Grant<AlarmData<A::Ticks>>) -> AlarmDriver<'a, A> {
        AlarmDriver {
            alarm: alarm,
            num_armed: Cell::new(0),
            app_alarms: grant,
            next_alarm: Cell::new(Expiration::Disabled),
        }
    }

    fn reset_active_alarm(&self) {
        let mut earliest_alarm = Expiration::Disabled;
        let mut earliest_end: A::Ticks = A::Ticks::from(0);
        let now = self.alarm.now();

        // Find the first alarm to fire and store it in earliest_alarm,
        // its counter value at earliest_end. In the case that there
        // are multiple alarms in the past, just store one of them
        // and resolve ordering later, when we fire.
        for alarm in self.app_alarms.iter() {
            alarm.enter(|alarm, _| match alarm.expiration {
                Expiration::Enabled { reference, dt } => {
                    // Do this because `reference` is shadowed below
                    let current_reference = reference;
                    let current_dt = dt;
                    let current_end = current_reference.wrapping_add(current_dt);

                    earliest_alarm = match earliest_alarm {
                        Expiration::Disabled => {
                            earliest_end = current_end;
                            alarm.expiration
                        }
                        Expiration::Enabled { reference, dt } => {
                            // There are two cases when current might be
                            // an earlier alarm.  The first is if it
                            // fires inside the interval (reference,
                            // reference+dt) of the existing earliest.
                            // The second is if now is not within the
                            // interval: this means that it has
                            // passed. It could be the earliest has passed
                            // too, but at this point we don't need to track
                            // which is earlier: the key point is that
                            // the alarm must fire immediately, and then when
                            // we handle the alarm callback the userspace
                            // callbacks will all be pushed onto processes.
                            // Because there is at most a single callback per
                            // process and they must go through the scheduler
                            // we don't care about the order in which we push
                            // their callbacks, as their order of execution is
                            // determined by the scheduler not push order. -pal
                            let temp_earliest_reference = reference;
                            let temp_earliest_dt = dt;
                            let temp_earliest_end =
                                temp_earliest_reference.wrapping_add(temp_earliest_dt);

                            if current_end.within_range(temp_earliest_reference, temp_earliest_end)
                            {
                                earliest_end = current_end;
                                alarm.expiration
                            } else if !now.within_range(temp_earliest_reference, temp_earliest_end)
                            {
                                earliest_end = temp_earliest_end;
                                alarm.expiration
                            } else {
                                earliest_alarm
                            }
                        }
                    }
                }
                Expiration::Disabled => {}
            });
        }
        self.next_alarm.set(earliest_alarm);
        match earliest_alarm {
            Expiration::Disabled => {
                self.alarm.disarm();
            }
            Expiration::Enabled { reference, dt } => {
                self.alarm.set_alarm(reference, dt);
            }
        }
    }
}

impl<'a, A: Alarm<'a>> Driver for AlarmDriver<'a, A> {
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
        self.app_alarms
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
    /// - `5`: Set an alarm to fire at a given clock value `time` relative to `now` (EXPERIMENTAL).
    fn command(&self, cmd_type: usize, data: usize, data2: usize, caller_id: AppId) -> ReturnCode {
        // Returns the error code to return to the user and whether we need to
        // reset which is the next active alarm. We _don't_ reset if
        //   - we're disabling the underlying alarm anyway,
        //   - the underlying alarm is currently disabled and we're enabling the first alarm, or
        //   - on an error (i.e. no change to the alarms).
        self.app_alarms
            .enter(caller_id, |td, _alloc| {
                // helper function to rearm alarm
                let rearm = |reference: A::Ticks, dt: A::Ticks| {
                    if let Expiration::Disabled = td.expiration {
                        self.num_armed.set(self.num_armed.get() + 1);
                    }
		    (
			Some(Expiration::Enabled {
                            reference,
                            dt,
			}),
                        ReturnCode::SuccessWithValue {
                            value: reference.wrapping_add(dt).into_u32() as usize,
                        },
                        true,
                    )
                };
                let now = self.alarm.now();
                let (new_expire, return_code, reset) = match cmd_type {
                    0 /* check if present */ => (None, ReturnCode::SuccessWithValue { value: 1 }, false),
                    1 /* Get clock frequency */ => {
                        let freq = <A::Frequency>::frequency() as usize;
                        (None, ReturnCode::SuccessWithValue { value: freq }, false)
                    },
                    2 /* capture time */ => {
                        (None, ReturnCode::SuccessWithValue { value: now.into_u32() as usize },
                         false)
                    },
                    3 /* Stop */ => {
                        match td.expiration {
                            Expiration::Disabled => {
                                // Request to stop when already stopped
                                (None, ReturnCode::EALREADY, false)
                            },
                            _ => {
                                td.expiration = Expiration::Disabled;
                                let new_num_armed = self.num_armed.get() - 1;
                                self.num_armed.set(new_num_armed);
                                (Some(Expiration::Disabled), ReturnCode::SUCCESS, true)
                            }
                        }
                    },
                    4 /* Set absolute expiration */ => {
			// To set an absolute expiration value, the
			// app delta is calculated with respect to the
			// 32-bit scaled kernel time and then used to
			// set the alarm respectively
			//
			// If the delta is too short (the alarm time
			// has just passed), this will delay the alarm
			// for a maximum of u32::MAX ticks
			let kernel_reference: A::Ticks = now;
			let app_alarm_time: Ticks32 = Ticks32::from(data as u32);
			let app_dt: Ticks32 = app_alarm_time.wrapping_sub(kernel_reference.into_u32().into());
			let kernel_dt: A::Ticks = A::Ticks::from(app_dt.into_u32());
                        // if previously unarmed, but now will become armed
                        rearm(kernel_reference, kernel_dt)
                    },
                    5 /* Set relative expiration */ => {
                        let reference = now;
                        let dt = A::Ticks::from(data as u32);

                        // if previously unarmed, but now will become armed
                        rearm(reference, dt)
                    },
                    6 /* Set absolute expiration with reference point */ => {
			// To set an alarm with an absolute expiration
			// value and given reference point, the app
			// reference is rebased onto the kernel
			// reference and used to calculate a kernel
			// delta.
			//
			// If the the alarm time has already passed
			// (judging based on the app reference), the
			// alarm will fire immediately.
			let kernel_reference: A::Ticks = now;
			let kernel_reference_appwidth: Ticks32 = Ticks32::from(kernel_reference.into_u32());
                        let app_reference = Ticks32::from(data as u32);
                        let app_dt = Ticks32::from(data2 as u32);

			if kernel_reference_appwidth.within_range(app_reference, app_reference.wrapping_add(app_dt)) {
			    // The alarm is yet to fire, calculate the
			    // delta with respect to the kernel
			    // reference and set it
			    let kernel_dt = A::Ticks::from(app_dt.wrapping_sub(kernel_reference_appwidth.wrapping_sub(app_reference)).into_u32());
			    rearm(kernel_reference, kernel_dt)
			} else {
			    // The kernel reference is outside of the
			    // app's alarm reference-delta interval,
			    // hence it must've already passed
			    //
			    // Fire an alarm immediately
                            rearm(kernel_reference, 0.into())
			}
                    }
                    _ => (None, ReturnCode::ENOSUPPORT, false)
                };
		if let Some(expire) = new_expire {
		    td.expiration = expire;
		}
                if reset {
                    self.reset_active_alarm();
                }
                return_code
            })
            .unwrap_or_else(|err| err.into())
    }
}

impl<'a, A: Alarm<'a>> time::AlarmClient for AlarmDriver<'a, A> {
    fn alarm(&self) {
        let now: A::Ticks = self.alarm.now();
        self.app_alarms.each(|alarm| {
            if let Expiration::Enabled { reference, dt } = alarm.expiration {
                // Now is not within reference, reference + ticks; this timer
                // has passed (since reference must be in the past)
                if !now.within_range(reference, reference.wrapping_add(dt)) {
                    alarm.expiration = Expiration::Disabled;
                    self.num_armed.set(self.num_armed.get() - 1);
                    alarm.callback.map(|mut cb| {
                        cb.schedule(
                            now.into_u32() as usize,
                            reference.wrapping_add(dt).into_u32() as usize,
                            0,
                        )
                    });
                }
            }
        });

        // If there are no armed alarms left, skip checking and just disable.
        // Otherwise, check all the alarms and find the next one, rescheduling
        // the underlying alarm.
        if self.num_armed.get() == 0 {
            self.alarm.disarm();
        } else {
            self.reset_active_alarm();
        }
    }
}
