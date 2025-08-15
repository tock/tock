// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Tock syscall driver capsule for Alarms, which issue callbacks when
//! a point in time has been reached.

use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::hil::time::{self, Alarm, Ticks};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Alarm as usize;

#[derive(Copy, Clone, Debug)]
struct Expiration<T: Ticks> {
    reference: T,
    dt: T,
}

#[derive(Copy, Clone)]
pub struct AlarmData<T: Ticks> {
    expiration: Option<Expiration<T>>,
}

const ALARM_CALLBACK_NUM: usize = 0;
const NUM_UPCALLS: u8 = 1;

impl<T: Ticks> Default for AlarmData<T> {
    fn default() -> AlarmData<T> {
        AlarmData { expiration: None }
    }
}

pub struct AlarmDriver<'a, A: Alarm<'a>> {
    alarm: &'a A,
    app_alarms:
        Grant<AlarmData<A::Ticks>, UpcallCount<NUM_UPCALLS>, AllowRoCount<0>, AllowRwCount<0>>,
}

impl<'a, A: Alarm<'a>> AlarmDriver<'a, A> {
    pub const fn new(
        alarm: &'a A,
        grant: Grant<
            AlarmData<A::Ticks>,
            UpcallCount<NUM_UPCALLS>,
            AllowRoCount<0>,
            AllowRwCount<0>,
        >,
    ) -> AlarmDriver<'a, A> {
        AlarmDriver {
            alarm,
            app_alarms: grant,
        }
    }

    /// Find the earliest [`Expiration`] from an iterator of expirations.
    ///
    /// Each [`Expiration`] value is provided as a tuple, with
    /// - `UD`: an additional user-data argument returned together with the
    ///   [`Expiration`], should it be the earliest, and
    /// - `F`: a call-back function invoked when the [`Expiration`] has already
    ///   expired. The callback is porivded the expiration value and a reference
    ///   to its user-data.
    ///
    /// Whether an [`Expiration`] has expired or not is determined with respect
    /// to `now`. If `now` is in `[exp.reference; exp.reference + exp.dt)`, it
    /// the [`Expiration`] has not yet expired.
    ///
    /// An expired [`Expiration`] is not a candidate for "earliest" expiration.
    /// This means that this function will return `Ok(None)` if it receives an
    /// empty iterator, or all [`Expiration`]s have expired.
    ///
    /// To stop iteration on any expired [`Expiration`], its callback can return
    /// `Some(R)`. Then this function will return `Err(Expiration, UD, R)`.
    /// This avoids consuming the entire iterator.
    fn earliest_alarm<UD, R, F: FnOnce(Expiration<A::Ticks>, &UD) -> Option<R>>(
        now: A::Ticks,
        expirations: impl Iterator<Item = (Expiration<A::Ticks>, UD, F)>,
    ) -> Result<Option<(Expiration<A::Ticks>, UD)>, (Expiration<A::Ticks>, UD, R)> {
        let mut earliest: Option<(Expiration<A::Ticks>, UD)> = None;

        for (exp, ud, expired_handler) in expirations {
            let Expiration {
                reference: exp_ref,
                dt: exp_dt,
            } = exp;

            // Pre-compute the absolute "end" time of this expiration (the
            // point at which it should fire):
            let exp_end = exp_ref.wrapping_add(exp_dt);

            // If `now` is not within `[reference, reference + dt)`, this
            // alarm has expired. Call the expired handler. If it returns
            // false, stop here.
            if !now.within_range(exp_ref, exp_end) {
                let expired_handler_res = expired_handler(exp, &ud);
                if let Some(retval) = expired_handler_res {
                    return Err((exp, ud, retval));
                }
            }

            // `exp` has not yet expired. At this point we can assume that
            // `now` is within `[exp_ref, exp_end)`. Check whether it will
            // expire earlier than the current `earliest`:
            match &earliest {
                None => {
                    // Do not have an earliest expiration yet, set this
                    // expriation as earliest:
                    earliest = Some((exp, ud));
                }

                Some((
                    Expiration {
                        reference: earliest_ref,
                        dt: earliest_dt,
                    },
                    _,
                )) => {
                    // As `now` is within `[ref, end)` for both timers, we
                    // can check which end time is closer to `now`. We thus
                    // first compute the end time for the current earliest
                    // alarm as well ...
                    let earliest_end = earliest_ref.wrapping_add(*earliest_dt);

                    // ... and then perform a wrapping_sub against `now` for
                    // both, checking which is smaller:
                    let exp_remain = exp_end.wrapping_sub(now);
                    let earliest_remain = earliest_end.wrapping_sub(now);
                    if exp_remain < earliest_remain {
                        // Our current exp expires earlier than earliest,
                        // replace it:
                        earliest = Some((exp, ud));
                    }
                }
            }
        }

        // We have computed earliest by iterating over all alarms, but have not
        // found one that has already expired. As such return `false`:
        Ok(earliest)
    }

    /// Re-arm the timer. This must be called in response to the underlying
    /// timer firing, or the set of [`Expiration`]s changing. This will iterate
    /// over all [`Expiration`]s and
    ///
    /// - invoke upcalls for all expired app alarms, resetting them afterwards,
    /// - re-arming the alarm for the next earliest [`Expiration`], or
    /// - disarming the alarm if no unexpired [`Expiration`] is found.
    fn process_rearm_or_callback(&self) {
        // Ask the clock about a current reference once. This can incur a
        // volatile read, and this may not be optimized if done in a loop:
        let now = self.alarm.now();

        let expired_handler = |expired: Expiration<A::Ticks>, process_id: &ProcessId| {
            // This closure is run on every expired alarm, _after_ the `enter()`
            // closure on the Grant iterator has returned. We are thus not
            // risking reentrancy here.

            // Enter the app's grant again:
            let _ = self.app_alarms.enter(*process_id, |alarm_state, upcalls| {
                // Reset this app's alarm:
                alarm_state.expiration = None;

                // Deliver the upcall:
                let _ = upcalls.schedule_upcall(
                    ALARM_CALLBACK_NUM,
                    (
                        now.into_u32_left_justified() as usize,
                        expired
                            .reference
                            .wrapping_add(expired.dt)
                            .into_u32_left_justified() as usize,
                        0,
                    ),
                );
            });

            // Proceed iteration across expirations:
            None::<()>
        };

        // Compute the earliest alarm, and invoke the `expired_handler` for
        // every expired alarm. This will issue a callback and reset the alarms
        // respectively.
        let res = Self::earliest_alarm(
            now,
            // Pass an interator of all non-None expirations:
            self.app_alarms.iter().filter_map(|app| {
                let process_id = app.processid();
                app.enter(|alarm_state, _upcalls| {
                    if let Some(exp) = alarm_state.expiration {
                        Some((exp, process_id, expired_handler))
                    } else {
                        None
                    }
                })
            }),
        );

        // Arm or disarm the alarm accordingly:
        match res {
            // No pending alarm, disarm:
            Ok(None) => {
                let _ = self.alarm.disarm();
            }

            // A future, non-expired alarm should fire:
            Ok(Some((Expiration { reference, dt }, _))) => {
                self.alarm.set_alarm(reference, dt);
            }

            // The expired closure has requested to stop iteration. This should
            // be unreachable, and hence we panic:
            Err((_, _, ())) => {
                unreachable!();
            }
        }
    }

    fn rearm_u32_left_justified_expiration(
        now: A::Ticks,
        reference_u32: Option<u32>,
        dt_u32: u32,
        expiration: &mut Option<Expiration<A::Ticks>>,
    ) -> u32 {
        let reference_unshifted = reference_u32.map(|ref_u32| ref_u32 >> A::Ticks::u32_padding());

        // If the underlying timer is less than 32-bit wide, userspace is able
        // to provide a finer `reference` and `dt` resolution than we can
        // possibly represent in the kernel.
        //
        // We do not want to switch back to userspace *before* the timer
        // fires. As such, when userspace gives us reference and ticks values
        // with a precision unrepresentible using our Ticks object, we round
        // `reference` down, and `dt` up (ensuring that the timer cannot fire
        // earlier than requested).
        let dt_unshifted = if let Some(reference_u32) = reference_u32 {
            // Computing unshifted dt for a userspace alarm can
            // underestimate dt in some cases where both reference and
            // dt had low-order bits that are rounded off by
            // unshifting. To ensure `dt` results in an actual
            // expiration that is at least as long as the expected
            // expiration in user space, compute unshifted dt from an
            // unshifted expiration.
            let expiration_shifted = reference_u32.wrapping_add(dt_u32);
            let expiration_unshifted =
                if expiration_shifted & ((1 << A::Ticks::u32_padding()) - 1) != 0 {
                    // By right-shifting, we would decrease the requested dt value,
                    // firing _before_ the time requested by userspace. Add one to
                    // compensate this:
                    (expiration_shifted >> A::Ticks::u32_padding()) + 1
                } else {
                    expiration_shifted >> A::Ticks::u32_padding()
                };

            expiration_unshifted.wrapping_sub(reference_u32 >> A::Ticks::u32_padding())
        } else if dt_u32 & ((1 << A::Ticks::u32_padding()) - 1) != 0 {
            // By right-shifting, we would decrease the requested dt value,
            // firing _before_ the time requested by userspace. Add one to
            // compensate this:
            (dt_u32 >> A::Ticks::u32_padding()) + 1
        } else {
            // dt does not need to be shifted *or* contains no lower bits
            // unrepresentable in the kernel:
            dt_u32 >> A::Ticks::u32_padding()
        };

        // For timers less than 32-bit wide, we do not have to handle a
        // `reference + dt` overflow specially. This is because those timers are
        // conveyed to us left-justified, and as such userspace would already
        // have to take care of such overflow.
        //
        // However, we *may* need to handle overflow when the timer is *wider*
        // than 32 bit. In this case, if `reference + dt` were to overflow, we
        // need to rebase our reference on the full-width `now` time.
        //
        // If userspace didn't give us a reference, we can skip all of this and
        // simply set the unshifted dt.
        let new_exp = match (reference_unshifted, A::Ticks::width() > 32) {
            (Some(userspace_reference_unshifted), true) => {
                // We have a userspace reference and timer is wider than 32 bit.
                //
                // In this case, we need to check whether the lower 32 bits of the
                // timer `reference` have already wrapped, compared to the reference
                // provided by userspace:
                if now.into_u32() < userspace_reference_unshifted {
                    // The lower 32-bit of reference are smaller than the userspace
                    // reference. This means that the full-width timer has had an
                    // increment in the upper bits. We thus set the full-width
                    // reference to the combination of the current upper timer bits
                    // *minus 1*, concatenated to the user-space provided bits.
                    //
                    // Because we don't know the integer type of the Ticks object
                    // (just that it's larger than a u32), we:
                    //
                    // 1. subtract a full `u32::MAX + 1` to incur a downward wrap,
                    //    effectively subtracting `1` from the upper part,
                    // 2. subtract the lower `u32` bits from this value, setting
                    //    those bits to zero,
                    // 3. adding back the userspace-provided reference.

                    // Build 1 << 32:
                    let bit33 = A::Ticks::from(0xffffffff).wrapping_add(A::Ticks::from(0x1));

                    // Perform step 1, subtracting 1 << 32:
                    let sub_1_upper = now.wrapping_sub(bit33);

                    // Perform step 2, setting first 32 bit to zero:
                    let sub_lower =
                        sub_1_upper.wrapping_sub(A::Ticks::from(sub_1_upper.into_u32()));

                    // Perform step 3, add back the userspace-provided reference:
                    let rebased_reference =
                        sub_lower.wrapping_add(A::Ticks::from(userspace_reference_unshifted));

                    // Finally, return the new expiration. We don't have to do
                    // anything special for `dt`, as it's relative:
                    Expiration {
                        reference: rebased_reference,
                        dt: A::Ticks::from(dt_unshifted),
                    }
                } else {
                    // The lower 32-bit of reference are equal to or larger than the
                    // userspace reference. Thus we can rebase the reference,
                    // touching only the lower 32 bit, by:
                    //
                    // 1. subtract the lower `u32` bits from this value, setting
                    //    those bits to zero,
                    // 2. adding back the userspace-provided reference.

                    // Perform step 1, setting first 32 bit to zero:
                    let sub_lower = now.wrapping_sub(A::Ticks::from(now.into_u32()));

                    // Perform step 2, add back the userspace-provided reference:
                    let rebased_reference =
                        sub_lower.wrapping_add(A::Ticks::from(userspace_reference_unshifted));

                    // Finally, return the new expiration. We don't have to do
                    // anything special for `dt`, as it's relative:
                    Expiration {
                        reference: rebased_reference,
                        dt: A::Ticks::from(dt_unshifted),
                    }
                }
            }

            (Some(userspace_reference_unshifted), false) => {
                // We have a userspace reference and timer is (less than) 32
                // bit. Simply set to unshifted values:

                Expiration {
                    reference: A::Ticks::from(userspace_reference_unshifted),
                    dt: A::Ticks::from(dt_unshifted),
                }
            }

            (None, _) => {
                // We have no userspace reference. Use `now` as a reference:
                Expiration {
                    reference: now,
                    dt: A::Ticks::from(dt_unshifted),
                }
            }
        };

        // Store the new expiration. We already adjusted the armed count above:
        *expiration = Some(new_exp);

        // Return the time left-justified time at which the alarm will fire:
        new_exp
            .reference
            .wrapping_add(new_exp.dt)
            .into_u32_left_justified()
    }
}

impl<'a, A: Alarm<'a>> SyscallDriver for AlarmDriver<'a, A> {
    /// Setup and read the alarm.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Driver existence check.
    /// - `1`: Return the clock frequency in Hz.
    /// - `2`: Read the current clock value
    /// - `3`: Stop the alarm if it is outstanding
    /// - `4`: Deprecated
    /// - `5`: Set an alarm to fire at a given clock value `time` relative to `now`
    /// - `6`: Set an alarm to fire at a given clock value `time` relative to a
    ///   provided reference point.
    fn command(
        &self,
        cmd_type: usize,
        data: usize,
        data2: usize,
        caller_id: ProcessId,
    ) -> CommandReturn {
        // Returns the error code to return to the user and whether we need to
        // reset which is the next active alarm. We _don't_ reset if
        //   - we're disabling the underlying alarm anyway,
        //   - the underlying alarm is currently disabled and we're enabling the first alarm, or
        //   - on an error (i.e. no change to the alarms).
        self.app_alarms
            .enter(caller_id, |td, _upcalls| {
                let now = self.alarm.now();

                match cmd_type {
                    // Driver check:
                    //
                    // Don't re-arm the timer:
                    0 => (CommandReturn::success(), false),

                    1 => {
                        // Get clock frequency. We return a frequency scaled by
                        // the amount of padding we add to the `ticks` value
                        // returned in command 2 ("capture time"), such that
                        // userspace knows when the timer will wrap and can
                        // accurately determine the duration of a single tick.
                        //
                        // Don't re-arm the timer:
                        let scaled_freq =
                            <A::Ticks>::u32_left_justified_scale_freq::<A::Frequency>();
                        (CommandReturn::success_u32(scaled_freq), false)
                    }
                    2 => {
                        // Capture time. We pad the underlying timer's ticks to
                        // wrap at exactly `(2 ** 32) - 1`. This predictable
                        // wrapping value allows userspace to build long running
                        // timers beyond `2 ** now.width()` ticks.
                        //
                        // Don't re-arm the timer:
                        (
                            CommandReturn::success_u32(now.into_u32_left_justified()),
                            false,
                        )
                    }
                    3 => {
                        // Stop
                        match td.expiration {
                            None => {
                                // Request to stop when already stopped. Don't
                                // re-arm the timer:
                                (CommandReturn::failure(ErrorCode::ALREADY), false)
                            }
                            Some(_old_expiraton) => {
                                // Clear the expiration:
                                td.expiration = None;

                                // Ask for the timer to be re-armed. We can't do
                                // this here, as it would re-enter the grant
                                // region:
                                (CommandReturn::success(), true)
                            }
                        }
                    }
                    4 => {
                        // Deprecated in 2.0, used to be: set absolute expiration
                        //
                        // Don't re-arm the timer:
                        (CommandReturn::failure(ErrorCode::NOSUPPORT), false)
                    }
                    5 => {
                        // Set relative expiration.
                        //
                        // We provided userspace a potentially padded version of
                        // our in-kernel Ticks object, and as such we have to
                        // invert that operation through a right shift.
                        //
                        // Also, we need to keep track of the currently armed
                        // timers.
                        //
                        // All of this is done in the following helper method:
                        let new_exp_left_justified = Self::rearm_u32_left_justified_expiration(
                            // Current time:
                            now,
                            // No userspace-provided reference:
                            None,
                            // Left-justified `dt` value:
                            data as u32,
                            // Reference to the `Option<Expiration>`, also used
                            // to update the counter of armed alarms:
                            &mut td.expiration,
                        );

                        // Report success, with the left-justified time at which
                        // the alarm will fire. Also ask for the timer to be
                        // re-armed. We can't do this here, as it would re-enter
                        // the grant region:
                        (CommandReturn::success_u32(new_exp_left_justified), true)
                    }
                    6 => {
                        // Also, we need to keep track of the currently armed
                        // timers.
                        //
                        // All of this is done in the following helper method:
                        let new_exp_left_justified = Self::rearm_u32_left_justified_expiration(
                            // Current time:
                            now,
                            // Left-justified userspace-provided reference:
                            Some(data as u32),
                            // Left-justified `dt` value:
                            data2 as u32,
                            // Reference to the `Option<Expiration>`, also used
                            // to update the counter of armed alarms:
                            &mut td.expiration,
                        );

                        // Report success, with the left-justified time at which
                        // the alarm will fire. Also ask for the timer to be
                        // re-armed. We can't do this here, as it would re-enter
                        // the grant region:
                        (CommandReturn::success_u32(new_exp_left_justified), true)
                    }

                    // Unknown command:
                    //
                    // Don't re-arm the timer:
                    _ => (CommandReturn::failure(ErrorCode::NOSUPPORT), false),
                }
            })
            .map_or_else(
                |err| CommandReturn::failure(err.into()),
                |(retval, rearm_timer)| {
                    if rearm_timer {
                        self.process_rearm_or_callback();
                    }
                    retval
                },
            )
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.app_alarms.enter(processid, |_, _| {})
    }
}

impl<'a, A: Alarm<'a>> time::AlarmClient for AlarmDriver<'a, A> {
    fn alarm(&self) {
        self.process_rearm_or_callback();
    }
}

#[cfg(test)]
mod test {
    use core::cell::Cell;
    use core::marker::PhantomData;

    use kernel::hil::time::{
        Alarm, AlarmClient, Freq10MHz, Frequency, Ticks, Ticks24, Ticks32, Ticks64, Time,
    };
    use kernel::utilities::cells::OptionalCell;
    use kernel::ErrorCode;

    use super::{AlarmDriver, Expiration};

    struct MockAlarm<'a, T: Ticks, F: Frequency> {
        current_ticks: Cell<T>,
        client: OptionalCell<&'a dyn AlarmClient>,
        _frequency: PhantomData<F>,
    }

    impl<T: Ticks, F: Frequency> Time for MockAlarm<'_, T, F> {
        type Frequency = F;
        type Ticks = T;

        fn now(&self) -> Self::Ticks {
            self.current_ticks.get()
        }
    }

    impl<'a, T: Ticks, F: Frequency> Alarm<'a> for MockAlarm<'a, T, F> {
        fn set_alarm_client(&self, client: &'a dyn AlarmClient) {
            self.client.set(client);
        }

        fn set_alarm(&self, _reference: Self::Ticks, _dt: Self::Ticks) {
            unimplemented!()
        }

        fn get_alarm(&self) -> Self::Ticks {
            unimplemented!()
        }

        fn disarm(&self) -> Result<(), ErrorCode> {
            unimplemented!()
        }

        fn is_armed(&self) -> bool {
            unimplemented!()
        }

        fn minimum_dt(&self) -> Self::Ticks {
            unimplemented!()
        }
    }

    #[test]
    fn test_earliest_alarm_no_alarms() {
        assert!(
            AlarmDriver::<MockAlarm<Ticks32, Freq10MHz>>::earliest_alarm(
                // Now:
                Ticks32::from(42_u32),
                // Expirations:
                <[(
                    Expiration<kernel::hil::time::Ticks32>,
                    (),
                    fn(_, &()) -> Option<()>
                ); 0] as IntoIterator>::into_iter([])
            )
            .unwrap()
            .is_none()
        )
    }

    #[test]
    fn test_earliest_alarm_multiple_unexpired() {
        // Should never be called:
        let exp_handler = |exp, id: &usize| -> Option<()> {
            panic!("Alarm should not be expired: {:?}, id: {}", exp, id)
        };

        let (earliest, id) = AlarmDriver::<MockAlarm<Ticks32, Freq10MHz>>::earliest_alarm(
            // Now:
            42_u32.into(),
            // Expirations:
            [
                (
                    // Will expire at 52:
                    Expiration {
                        reference: 42_u32.into(),
                        dt: 10_u32.into(),
                    },
                    0,
                    exp_handler,
                ),
                (
                    // Will expire at exactly 43:
                    Expiration {
                        reference: u32::MAX.into(),
                        dt: 44_u32.into(),
                    },
                    1,
                    exp_handler,
                ),
                (
                    // Will expire at 44:
                    Expiration {
                        reference: 10_u32.into(),
                        dt: 34_u32.into(),
                    },
                    2,
                    exp_handler,
                ),
            ]
            .into_iter(),
        )
        .unwrap()
        .unwrap();

        assert!(earliest.reference.into_u32() == u32::MAX);
        assert!(earliest.dt.into_u32() == 44);
        assert!(id == 1);
    }

    #[test]
    fn test_earliest_alarm_multiple_expired() {
        let exp_list: [Cell<bool>; 7] = Default::default();

        let exp_handler = |_exp, id: &usize| -> Option<()> {
            exp_list[*id].set(true);

            // Don't stop iterating on the first expired alarm:
            None
        };

        let (earliest, id) = AlarmDriver::<MockAlarm<Ticks32, Freq10MHz>>::earliest_alarm(
            // Now:
            42_u32.into(),
            // Expirations:
            [
                (
                    // Has expired at 42 (current cycle), should fire!
                    Expiration {
                        reference: 41_u32.into(),
                        dt: 1_u32.into(),
                    },
                    0,
                    &exp_handler,
                ),
                (
                    // Will expire at 52, should not fire.
                    Expiration {
                        reference: 42_u32.into(),
                        dt: 10_u32.into(),
                    },
                    1,
                    &exp_handler,
                ),
                (
                    // Will expire at exactly 43, should not fire.
                    Expiration {
                        reference: u32::MAX.into(),
                        dt: 44_u32.into(),
                    },
                    2,
                    &exp_handler,
                ),
                (
                    // Reference is current time, expiration in the future,
                    // should not fire:
                    Expiration {
                        reference: 42_u32.into(),
                        dt: 1_u32.into(),
                    },
                    3,
                    &exp_handler,
                ),
                (
                    // Reference is 43 (current time + 1), interpreted as "in
                    // the past", should fire:
                    Expiration {
                        reference: 43_u32.into(),
                        dt: 1_u32.into(),
                    },
                    4,
                    &exp_handler,
                ),
                (
                    // Reference is 0, end is at 1, in the past, should fire:
                    Expiration {
                        reference: 0_u32.into(),
                        dt: 1_u32.into(),
                    },
                    5,
                    &exp_handler,
                ),
                (
                    // Reference is u32::MAX, end is at 0, in the past, should fire:
                    Expiration {
                        reference: u32::MAX.into(),
                        dt: 1_u32.into(),
                    },
                    6,
                    &exp_handler,
                ),
            ]
            .into_iter(),
        )
        .unwrap()
        .unwrap();

        assert!(earliest.reference.into_u32() == 41);
        assert!(earliest.dt.into_u32() == 1);
        assert!(id == 0);

        let mut bool_exp_list: [bool; 7] = [false; 7];
        exp_list
            .into_iter()
            .zip(bool_exp_list.iter_mut())
            .for_each(|(src, dst)| *dst = src.get());

        assert!(bool_exp_list == [true, false, false, false, true, true, true]);
    }

    #[test]
    fn test_earliest_alarm_expired_stop() {
        let exp_list: [Cell<bool>; 4] = Default::default();

        let exp_handler = |_exp, id: &usize| -> Option<&'static str> {
            exp_list[*id].set(true);

            // Stop iterating on id == 3
            if *id == 3 {
                Some("stopped")
            } else {
                None
            }
        };

        let (expired, id, expired_ret) =
            AlarmDriver::<MockAlarm<Ticks32, Freq10MHz>>::earliest_alarm(
                // Now:
                42_u32.into(),
                // Expirations:
                [
                    (
                        // Will expire at 52, should not fire.
                        Expiration {
                            reference: 42_u32.into(),
                            dt: 10_u32.into(),
                        },
                        0,
                        &exp_handler,
                    ),
                    (
                        // Has expired at 42 (current cycle), should fire!
                        Expiration {
                            reference: 41_u32.into(),
                            dt: 1_u32.into(),
                        },
                        1,
                        &exp_handler,
                    ),
                    (
                        // Will expire at exactly 43, should not fire.
                        Expiration {
                            reference: u32::MAX.into(),
                            dt: 44_u32.into(),
                        },
                        2,
                        &exp_handler,
                    ),
                    (
                        // Reference is 0, end is at 1, in the past, should fire:
                        Expiration {
                            reference: 0_u32.into(),
                            dt: 1_u32.into(),
                        },
                        3,
                        &exp_handler,
                    ),
                ]
                .into_iter(),
            )
            .err()
            .unwrap();

        assert!(expired.reference.into_u32() == 0);
        assert!(expired.dt.into_u32() == 1);
        assert!(id == 3);
        assert!(expired_ret == "stopped");

        let mut bool_exp_list: [bool; 4] = [false; 4];
        exp_list
            .into_iter()
            .zip(bool_exp_list.iter_mut())
            .for_each(|(src, dst)| *dst = src.get());

        assert!(bool_exp_list == [false, true, false, true,]);
    }

    #[test]
    fn test_rearm_24bit_left_justified_noref_basic() {
        let mut expiration = None;

        assert!(Ticks24::u32_padding() == 8);

        let armed_time =
            AlarmDriver::<MockAlarm<Ticks24, Freq10MHz>>::rearm_u32_left_justified_expiration(
                // Current time:
                Ticks24::from(1337_u32),
                // No userspace-provided reference:
                None,
                // Left-justified `dt` value:
                1234_u32 << Ticks24::u32_padding(),
                // Reference to the `Option<Expiration>`, also used
                // to update the counter of armed alarms:
                &mut expiration,
            );

        let expiration = expiration.unwrap();

        assert_eq!(armed_time, (1337 + 1234) << Ticks24::u32_padding());
        assert_eq!(expiration.reference.into_u32(), 1337);
        assert_eq!(expiration.dt.into_u32(), 1234);
    }

    #[test]
    fn test_rearm_24bit_left_justified_noref_wrapping() {
        let mut expiration = None;

        let armed_time =
            AlarmDriver::<MockAlarm<Ticks24, Freq10MHz>>::rearm_u32_left_justified_expiration(
                // Current time:
                Ticks24::from(1337_u32),
                // No userspace-provided reference:
                None,
                // Left-justified `dt` value (in this case, with some
                // irrepresentable precision)
                u32::MAX - (42 << Ticks24::u32_padding()),
                // Reference to the `Option<Expiration>`, also used
                // to update the counter of armed alarms:
                &mut expiration,
            );

        let expiration = expiration.unwrap();

        // (1337 + ((0xffffffff - (42 << 8)) >> 8) + 1) % 0x01000000 = 1295
        assert_eq!(armed_time, 1295 << Ticks24::u32_padding());
        assert_eq!(expiration.reference.into_u32(), 1337);
        assert_eq!(
            expiration.dt.into_u32(),
            // dt is rounded up to the next representable tick:
            ((u32::MAX - (42 << Ticks24::u32_padding())) >> Ticks24::u32_padding()) + 1
        );
    }

    #[test]
    fn test_rearm_24bit_left_justified_ref_low_bits_basic() {
        let mut expiration = None;

        assert!(Ticks24::u32_padding() == 8);

        let armed_time =
            AlarmDriver::<MockAlarm<Ticks24, Freq10MHz>>::rearm_u32_left_justified_expiration(
                // Current time:
                Ticks24::from(0_u32),
                // Userspace-provided reference, below minimum precision of Ticks24, will be rounded down:
                Some(1_u32),
                // Left-justified `dt` value, below minimum precision of Ticks24, will be rounded up:
                3_u32,
                // Reference to the `Option<Expiration>`, also used
                // to update the counter of armed alarms:
                &mut expiration,
            );

        let expiration = expiration.unwrap();

        // ((1 >> 8) + ((3 >> 8) + 1)  << 8) = 1
        assert_eq!(armed_time, 1 << 8);
        assert_eq!(expiration.reference.into_u32(), 0);
        assert_eq!(expiration.dt.into_u32(), 1);
    }

    #[test]
    fn test_rearm_24bit_left_justified_ref_low_bits_max_int() {
        let mut expiration = None;

        assert!(Ticks24::u32_padding() == 8);

        let armed_time =
            AlarmDriver::<MockAlarm<Ticks24, Freq10MHz>>::rearm_u32_left_justified_expiration(
                // Current time:
                Ticks24::from(6_u32),
                // Userspace-provided reference, including bits not representable in Ticks24:
                // (5 << 8) - 43 = 1237
                Some(Ticks24::from(5_u32).into_u32_left_justified() - 43),
                // Left-justified `dt` value, including bits not representable in Ticks24:
                // (2 << 8) - 43 = 469
                Ticks24::from(2_u32).into_u32_left_justified() + 43,
                // Reference to the `Option<Expiration>`, also used
                // to update the counter of armed alarms:
                &mut expiration,
            );

        let expiration = expiration.unwrap();

        // When we naively round down reference `(1237 / 256 ~= 4.83 -> 4)` and
        // round up dt `(469 / 256 ~= 1.83 -> 2)` we'd arm the alarm to
        // `2 + 4 = 6`. However, when considering the full resolution
        // `reference + dt` `(1237 + 256) / 256 ~= 6.67` we can see that arming
        // to `6` will have the alarm fire too early. The alarm rearm code needs
        // to compensate for the case that (reference + dt) overflow and generate
        // a dt from that rounded value, in this case `7`.
        assert_eq!(armed_time, 7 << 8);
        assert_eq!(expiration.reference.into_u32(), 4);
        assert_eq!(expiration.dt, Ticks24::from(3));
    }

    #[test]
    fn test_rearm_32bit_left_justified_noref_basic() {
        let mut expiration = Some(Expiration {
            reference: 0_u32.into(),
            dt: 1_u32.into(),
        });

        assert!(Ticks32::u32_padding() == 0);

        let armed_time =
            AlarmDriver::<MockAlarm<Ticks32, Freq10MHz>>::rearm_u32_left_justified_expiration(
                // Current time:
                Ticks32::from(1337_u32),
                // No userspace-provided reference:
                None,
                // Left-justified `dt` value, unshifted for 32 bit:
                1234_u32,
                // Reference to the `Option<Expiration>`, also used
                // to update the counter of armed alarms:
                &mut expiration,
            );

        let expiration = expiration.unwrap();

        assert_eq!(armed_time, 1337 + 1234);
        assert_eq!(expiration.reference.into_u32(), 1337);
        assert_eq!(expiration.dt.into_u32(), 1234);
    }

    #[test]
    fn test_rearm_32bit_left_justified_noref_wrapping() {
        let mut expiration = None;

        let armed_time =
            AlarmDriver::<MockAlarm<Ticks32, Freq10MHz>>::rearm_u32_left_justified_expiration(
                // Current time:
                Ticks32::from(1337_u32),
                // No userspace-provided reference:
                None,
                // Left-justified `dt` value (in this case, with some
                // irrepresentable precision)
                u32::MAX - 42,
                // Reference to the `Option<Expiration>`, also used
                // to update the counter of armed alarms:
                &mut expiration,
            );

        let expiration = expiration.unwrap();

        // (1337 + (0xffffffff - 42)) % 0x100000000 = 1294
        assert_eq!(armed_time, 1294);
        assert_eq!(expiration.reference.into_u32(), 1337);
        assert_eq!(expiration.dt.into_u32(), u32::MAX - 42);
    }

    #[test]
    fn test_rearm_64bit_left_justified_noref_wrapping() {
        let mut expiration = Some(Expiration {
            reference: 0_u32.into(),
            dt: 1_u32.into(),
        });

        assert!(Ticks64::u32_padding() == 0);

        let armed_time =
            AlarmDriver::<MockAlarm<Ticks64, Freq10MHz>>::rearm_u32_left_justified_expiration(
                // Current time:
                Ticks64::from(0xDEADBEEFCAFE_u64),
                // No userspace-provided reference:
                None,
                // Left-justified `dt` value, unshifted for 32 bit:
                0xDEADC0DE_u32,
                // Reference to the `Option<Expiration>`, also used
                // to update the counter of armed alarms:
                &mut expiration,
            );

        let expiration = expiration.unwrap();

        assert_eq!(armed_time, 0x9D9D8BDC_u32);
        assert_eq!(expiration.reference.into_u64(), 0xDEADBEEFCAFE_u64);
        assert_eq!(expiration.dt.into_u64(), 0xDEADC0DE_u64);
    }

    #[test]
    fn test_rearm_64bit_left_justified_refnowrap_dtnorwap() {
        let mut expiration = None;

        // reference smaller than now & 0xffffffff, reference + dt don't wrap:
        let armed_time =
            AlarmDriver::<MockAlarm<Ticks64, Freq10MHz>>::rearm_u32_left_justified_expiration(
                // Current time:
                Ticks64::from(0xDEADBEEFCAFE_u64),
                // Userspace-provided reference, smaller than now and dt
                Some(0xBEEFC0DE_u32),
                // Left-justified `dt` value, unshifted for 32 bit:
                0x1BADB002_u32,
                // Reference to the `Option<Expiration>`, also used
                // to update the counter of armed alarms:
                &mut expiration,
            );

        let expiration = expiration.unwrap();

        assert_eq!(armed_time, 0xDA9D70E0_u32); // remains at 0xDEAD
        assert_eq!(expiration.reference.into_u64(), 0xDEADBEEFC0DE_u64);
        assert_eq!(expiration.dt.into_u64(), 0x1BADB002_u64);
    }

    #[test]
    fn test_rearm_64bit_left_justified_refnowrwap_dtwrap() {
        let mut expiration = Some(Expiration {
            reference: 0_u32.into(),
            dt: 1_u32.into(),
        });

        // reference smaller than now & 0xffffffff, reference + dt wrap:
        let armed_time =
            AlarmDriver::<MockAlarm<Ticks64, Freq10MHz>>::rearm_u32_left_justified_expiration(
                // Current time:
                Ticks64::from(0xDEADBEEFCAFE_u64),
                // Userspace-provided reference, smaller than lower 32-bit of now
                Some(0x8BADF00D_u32),
                // Left-justified `dt` value, unshifted for 32 bit:
                0xFEEDC0DE_u32,
                // Reference to the `Option<Expiration>`, also used
                // to update the counter of armed alarms:
                &mut expiration,
            );

        let expiration = expiration.unwrap();

        assert_eq!(armed_time, 0x8A9BB0EB_u32); // wraps t0 0x0xDEAE
        assert_eq!(expiration.reference.into_u64(), 0xDEAD8BADF00D_u64);
        assert_eq!(expiration.dt.into_u64(), 0xFEEDC0DE_u64);
    }

    #[test]
    fn test_rearm_64bit_left_justified_refwrap_dtwrap() {
        let mut expiration = None;

        // reference larger than now & 0xffffffff, reference + dt wrap:
        let armed_time =
            AlarmDriver::<MockAlarm<Ticks64, Freq10MHz>>::rearm_u32_left_justified_expiration(
                // Current time:
                Ticks64::from(0xDEADBEEFCAFE_u64),
                // Userspace-provided reference, larger than lower 32-bit of
                // now, meaning that it's already past:
                Some(0xCAFEB0BA_u32),
                // Left-justified `dt` value, unshifted for 32 bit:
                0xFEEDC0DE_u32,
                // Reference to the `Option<Expiration>`, also used
                // to update the counter of armed alarms:
                &mut expiration,
            );

        let expiration = expiration.unwrap();

        assert_eq!(armed_time, 0xC9EC7198_u32); // wraps to 0xDEAE
        assert_eq!(expiration.reference.into_u64(), 0xDEACCAFEB0BA_u64);
        assert_eq!(expiration.dt.into_u64(), 0xFEEDC0DE_u64);
    }

    #[test]
    fn test_rearm_64bit_left_justified_refwrap_dtnowrap() {
        let mut expiration = Some(Expiration {
            reference: 0_u32.into(),
            dt: 1_u32.into(),
        });

        // reference larger than now & 0xffffffff, reference + dt don't wrap
        let armed_time =
            AlarmDriver::<MockAlarm<Ticks64, Freq10MHz>>::rearm_u32_left_justified_expiration(
                // Current time:
                Ticks64::from(0xDEADBEEFCAFE_u64),
                // Userspace-provided reference, larger than lower 32-bit of now
                Some(0xCAFEB0BA_u32),
                // Left-justified `dt` value, unshifted for 32 bit:
                0x1BADB002_u32,
                // Reference to the `Option<Expiration>`, also used
                // to update the counter of armed alarms:
                &mut expiration,
            );

        let expiration = expiration.unwrap();

        assert_eq!(armed_time, 0xE6AC60BC_u32); // remains at 0xDEAD
        assert_eq!(expiration.reference.into_u64(), 0xDEACCAFEB0BA_u64);
        assert_eq!(expiration.dt.into_u64(), 0x1BADB002_u64);
    }
}
