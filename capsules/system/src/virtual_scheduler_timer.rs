// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Implementation of [`SchedulerTimer`] trait on top of a virtual alarm.

use core::num::NonZeroU32;
use kernel::hil::time::{self, Frequency, Ticks};
use kernel::platform::scheduler_timer::SchedulerTimer;

/// Implementation of [`SchedulerTimer`] trait on top of a virtual alarm.
///
/// Currently, this implementation depends slightly on the virtual alarm
/// implementation in capsules -- namely it assumes that get_alarm will still
/// return the passed value even after the timer is disarmed. Thus this should
/// only be implemented with a virtual alarm. If a dedicated hardware timer is
/// available, it is more performant to implement the scheduler timer directly
/// for that hardware peripheral without the alarm abstraction in between.
///
/// This mostly handles conversions from wall time, the required inputs to the
/// trait, to ticks, which are used to track time for alarms.
pub struct VirtualSchedulerTimer<A: 'static + time::Alarm<'static>> {
    alarm: &'static A,
}

impl<A: 'static + time::Alarm<'static>> VirtualSchedulerTimer<A> {
    pub fn new(alarm: &'static A) -> Self {
        Self { alarm }
    }
}

impl<A: 'static + time::Alarm<'static>> SchedulerTimer for VirtualSchedulerTimer<A> {
    fn reset(&self) {
        let _ = self.alarm.disarm();
    }

    fn start(&self, us: NonZeroU32) {
        let tics = {
            // We need to convert from microseconds to native tics, which could overflow in 32-bit
            // arithmetic. So we convert to 64-bit. 64-bit division is an expensive subroutine, but
            // if `us` is a power of 10 the compiler will simplify it with the 1_000_000 divisor
            // instead.
            let us = us.get() as u64;
            let hertz = A::Frequency::frequency() as u64;

            (hertz * us / 1_000_000) as u32
        };

        let reference = self.alarm.now();
        self.alarm.set_alarm(reference, A::Ticks::from(tics));
    }

    fn arm(&self) {
        //self.alarm.arm();
    }

    fn disarm(&self) {
        //self.alarm.disarm();
    }

    fn get_remaining_us(&self) -> Option<NonZeroU32> {
        // We need to convert from native tics to us, multiplication could overflow in 32-bit
        // arithmetic. So we convert to 64-bit.

        let diff = self
            .alarm
            .get_alarm()
            .wrapping_sub(self.alarm.now())
            .into_u32() as u64;

        // If next alarm is more than one second away from now, alarm must have expired.
        // Use this formulation to protect against errors when now has passed alarm.
        // 1 second was chosen because it is significantly greater than the 400ms max value allowed
        // by start(), and requires no computational overhead (e.g. using 500ms would require
        // dividing the returned ticks by 2)
        // However, if the alarm frequency is slow enough relative to the cpu frequency, it is
        // possible this will be evaluated while now() == get_alarm(), so we special case that
        // result where the alarm has fired but the subtraction has not overflowed
        if diff >= A::Frequency::frequency() as u64 {
            None
        } else {
            let hertz = A::Frequency::frequency() as u64;
            NonZeroU32::new(((diff * 1_000_000) / hertz) as u32)
        }
    }
}
