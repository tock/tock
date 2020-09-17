//! Provides userspace applications with a alarm API.

use core::cell::Cell;
use kernel::debug;
use kernel::hil::time::{self, Alarm, Frequency, Ticks};
use kernel::{AppId, Callback, Driver, Grant, ReturnCode};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Alarm as usize;

// This should transition to using Ticks
#[derive(Copy, Clone, Debug)]
enum Expiration {
    Disabled,
    Enabled { reference: u32, dt: u32 },
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
    app_alarms: Grant<AlarmData>,
    next_alarm: Cell<Expiration>,
}

impl<'a, A: Alarm<'a>> AlarmDriver<'a, A> {
    pub const fn new(alarm: &'a A, grant: Grant<AlarmData>) -> AlarmDriver<'a, A> {
        AlarmDriver {
            alarm: alarm,
            num_armed: Cell::new(0),
            app_alarms: grant,
            next_alarm: Cell::new(Expiration::Disabled),
        }
    }

    // This logic is tricky because it needs to handle the case when the
    // underlying alarm is wider than 32 bits.
    fn reset_active_alarm(&self) {
        let mut earliest_alarm = Expiration::Disabled;
        let mut earliest_end: A::Ticks = A::Ticks::from(0);
        // Scale now down to a u32 since that is the width of the alarm;
        // otherwise for larger widths (e.g., u64) now can be outside of
        // the range of what an alarm can be set to.
        let now = self.alarm.now();
        let now_lower_bits = A::Ticks::from(now.into_u32());
        // Find the first alarm to fire and store it in earliest_alarm,6
        // its counter value at earliest_end. In the case that there
        // are multiple alarms in the past, just store one of them
        // and resolve ordering later, when we fire.
        for alarm in self.app_alarms.iter() {
            alarm.enter(|alarm, _| match alarm.expiration {
                Expiration::Enabled { reference, dt } => {
                    // Do this because reference shadowed below
                    let current_reference = reference;
                    let current_reference_ticks = A::Ticks::from(current_reference);
                    let current_dt = dt;
                    let current_dt_ticks = A::Ticks::from(current_dt);
                    let current_end_ticks = current_reference_ticks.wrapping_add(current_dt_ticks);

                    earliest_alarm = match earliest_alarm {
                        Expiration::Disabled => {
                            earliest_end = current_end_ticks;
                            alarm.expiration
                        }
                        Expiration::Enabled { reference, dt } => {
                            // There are two cases when this might be
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
                            let temp_earliest_reference = A::Ticks::from(reference);
                            let temp_earliest_dt = A::Ticks::from(dt);
                            let temp_earliest_end =
                                temp_earliest_reference.wrapping_add(temp_earliest_dt);

                            if current_end_ticks
                                .within_range(temp_earliest_reference, temp_earliest_end)
                            {
                                earliest_end = current_end_ticks;
                                alarm.expiration
                            } else if !now_lower_bits
                                .within_range(temp_earliest_reference, temp_earliest_end)
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
        debug!("AlarmDriver: earliest: {:?}", earliest_alarm);
        self.next_alarm.set(earliest_alarm);
        match earliest_alarm {
            Expiration::Disabled => {
                self.alarm.disarm();
            }
            Expiration::Enabled { reference, dt } => {
                // This logic handles when the underlying Alarm is wider than
                // 32 bits; it sets the reference to include the high bits of now
                let mut high_bits = now.wrapping_sub(now_lower_bits);
                // If lower bits have wrapped around from reference, this means the
                // reference's high bits are actually one less; if we don't subtract
                // one then the alarm will incorrectly be set 1<<32 higher than it should.
                // This uses the invariant that reference <= now.
                if now_lower_bits.into_u32() < reference {
                    debug!(
                        "Incrementing bit 32: lower={}, reference={}",
                        now_lower_bits.into_u32(),
                        reference
                    );
                    // Build 1<<32 in a way that just overflows to 0 if we are 32 bits
                    let bit33 = A::Ticks::from(0xffffffff).wrapping_add(A::Ticks::from(0x1));
                    high_bits = high_bits.wrapping_sub(bit33);
                }
                let real_reference = high_bits.wrapping_add(A::Ticks::from(reference));
                self.alarm.set_alarm(real_reference, A::Ticks::from(dt));
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
                let mut rearm = |reference: usize, dt: usize| {
                    if let Expiration::Disabled = td.expiration {
                        self.num_armed.set(self.num_armed.get() + 1);
                    }
                    td.expiration = Expiration::Enabled {
                        reference: reference as u32,
                        dt: dt as u32,
                    };
                    (
                        ReturnCode::SuccessWithValue {
                            value: reference.wrapping_add(dt),
                        },
                        true,
                    )
                };
                let now = self.alarm.now();
                let (return_code, reset) = match cmd_type {
                    0 /* check if present */ => (ReturnCode::SuccessWithValue { value: 1 }, false),
                    1 /* Get clock frequency */ => {
                        let freq = <A::Frequency>::frequency() as usize;
                        (ReturnCode::SuccessWithValue { value: freq }, false)
                    },
                    2 /* capture time */ => {
                        (ReturnCode::SuccessWithValue { value: now.into_u32() as usize },
                         false)
                    },
                    3 /* Stop */ => {
                        match td.expiration {
                            Expiration::Disabled => {
                                // Request to stop when already stopped
                                (ReturnCode::EALREADY, false)
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
                        let reference = now.into_u32() as usize;
                        let future_time = data;
                        let dt = future_time.wrapping_sub(reference);
                        // if previously unarmed, but now will become armed
                        //debug!("Rearming alarm for {} + {}", reference, dt);
                        rearm(reference, dt)
                    },
                    5 /* Set relative expiration */ => {
                        let reference = now.into_u32() as usize;
                        let dt = data;
                        // if previously unarmed, but now will become armed
                        rearm(reference, dt)
                    },
                    6 /* Set absolute expiration with reference point */ => {
                        // Taking a reference timestamp from userspace
                        // prevents wraparound bugs; future versions of
                        // libtock will use only this call and deprecate
                        // command #4; for now it is added as an additional
                        // comamnd for backwards compatibility. -pal
                        let reference = data;
                        let dt = data2;
                        //debug!("Rearming alarm for {} + {} = {}", reference, dt, reference.wrapping_add(dt));
                        rearm(reference, dt)
                    }
                    _ => (ReturnCode::ENOSUPPORT, false)
                };
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
        //debug!("AlarmDriver::alarm called at {}", now.into_u32());

        self.app_alarms.each(|alarm| {
            if let Expiration::Enabled { reference, dt } = alarm.expiration {
                // Now is not within reference, reference + ticks; this timer
                // as passed (since reference must be in the past)
                if !now.within_range(
                    A::Ticks::from(reference),
                    A::Ticks::from(reference.wrapping_add(dt)),
                ) {
                    alarm.expiration = Expiration::Disabled;
                    self.num_armed.set(self.num_armed.get() - 1);
                    alarm.callback.map(|mut cb| {
                        cb.schedule(
                            now.into_u32() as usize,
                            reference.wrapping_add(dt) as usize,
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
