// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Virtualize the Alarm interface to enable multiple users of an underlying
//! alarm hardware peripheral.

use core::cell::Cell;

use kernel::collections::list::{List, ListLink, ListNode};
use kernel::hil::time::{self, Alarm, Ticks, Time};
use kernel::utilities::cells::OptionalCell;
use kernel::ErrorCode;

#[derive(Copy, Clone)]
struct TickDtReference<T: Ticks> {
    /// Reference time point when this alarm was setup.
    reference: T,
    /// Duration of this alarm w.r.t. the reference time point. In other words, this alarm should
    /// fire at `reference + dt`.
    dt: T,
    /// True if this dt only represents a portion of the original dt that was requested. If true,
    /// then we need to wait for another max_tick/2 after an internal extended dt reference alarm
    /// fires. This ensures we can wait the full max_tick even if there is latency in the system.
    extended: bool,
}

impl<T: Ticks> TickDtReference<T> {
    #[inline]
    fn reference_plus_dt(&self) -> T {
        self.reference.wrapping_add(self.dt)
    }
}

/// An object to multiplex multiple "virtual" alarms over a single underlying alarm. A
/// `VirtualMuxAlarm` is a node in a linked list of alarms that share the same underlying alarm.
pub struct VirtualMuxAlarm<'a, A: Alarm<'a>> {
    /// Underlying alarm which multiplexes all these virtual alarm.
    mux: &'a MuxAlarm<'a, A>,
    /// Reference and dt point when this alarm was setup.
    dt_reference: Cell<TickDtReference<A::Ticks>>,
    /// Whether this alarm is currently armed, i.e. whether it should fire when the time has
    /// elapsed.
    armed: Cell<bool>,
    /// Next alarm in the list.
    next: ListLink<'a, VirtualMuxAlarm<'a, A>>,
    /// Alarm client for this node in the list.
    client: OptionalCell<&'a dyn time::AlarmClient>,
}

impl<'a, A: Alarm<'a>> ListNode<'a, VirtualMuxAlarm<'a, A>> for VirtualMuxAlarm<'a, A> {
    fn next(&self) -> &'a ListLink<VirtualMuxAlarm<'a, A>> {
        &self.next
    }
}

impl<'a, A: Alarm<'a>> VirtualMuxAlarm<'a, A> {
    /// After calling new, always call setup()
    pub fn new(mux_alarm: &'a MuxAlarm<'a, A>) -> VirtualMuxAlarm<'a, A> {
        let zero = A::Ticks::from(0);
        VirtualMuxAlarm {
            mux: mux_alarm,
            dt_reference: Cell::new(TickDtReference {
                reference: zero,
                dt: zero,
                extended: false,
            }),
            armed: Cell::new(false),
            next: ListLink::empty(),
            client: OptionalCell::empty(),
        }
    }

    /// Call this method immediately after new() to link this to the mux, otherwise alarms won't
    /// fire
    pub fn setup(&'a self) {
        self.mux.virtual_alarms.push_head(self);
    }
}

impl<'a, A: Alarm<'a>> Time for VirtualMuxAlarm<'a, A> {
    type Frequency = A::Frequency;
    type Ticks = A::Ticks;

    fn now(&self) -> Self::Ticks {
        self.mux.alarm.now()
    }
}

impl<'a, A: Alarm<'a>> Alarm<'a> for VirtualMuxAlarm<'a, A> {
    fn set_alarm_client(&self, client: &'a dyn time::AlarmClient) {
        self.client.set(client);
    }

    fn disarm(&self) -> Result<(), ErrorCode> {
        if !self.armed.get() {
            return Ok(());
        }

        self.armed.set(false);

        let enabled = self.mux.enabled.get() - 1;
        self.mux.enabled.set(enabled);

        // If there are not more enabled alarms, disable the underlying alarm
        // completely.
        if enabled == 0 {
            let _ = self.mux.alarm.disarm();
        }
        Ok(())
    }

    fn is_armed(&self) -> bool {
        self.armed.get()
    }

    fn set_alarm(&self, reference: Self::Ticks, dt: Self::Ticks) {
        let enabled = self.mux.enabled.get();
        let half_max = Self::Ticks::half_max_value();
        // If the dt is more than half of the available time resolution, then we need to break
        // up the alarm into two internal alarms. This ensures that our internal comparisons of
        // now outside of range [ref, ref + dt) will trigger correctly even with latency in the
        // system
        let dt_reference = if dt > half_max.wrapping_add(self.minimum_dt()) {
            TickDtReference {
                reference,
                dt: dt.wrapping_sub(half_max),
                extended: true,
            }
        } else {
            TickDtReference {
                reference,
                dt,
                extended: false,
            }
        };
        self.dt_reference.set(dt_reference);
        // Ensure local variable has correct value when used below
        let dt = dt_reference.dt;

        if !self.armed.get() {
            self.mux.enabled.set(enabled + 1);
            self.armed.set(true);
        }

        // First alarm, so set it
        if enabled == 0 {
            //debug!("virtual_alarm: first alarm: set it.");
            self.mux.set_alarm(reference, dt);
        } else if !self.mux.firing.get() {
            // If firing is true, the mux will scan all the alarms after
            // firing and pick the soonest one so do not need to modify the
            // mux. Otherwise, this is an alarm
            // started in a separate code path (e.g., another event).
            // This new alarm fires sooner if two things are both true:
            //    1. The current earliest alarm expiration doesn't fall
            //    in the range of [reference, reference+dt): this means
            //    it is either in the past (before reference) or the future
            //    (reference + dt), AND
            //    2. now falls in the [reference, reference+dt)
            //    window of the current earliest alarm. This means the
            //    current earliest alarm hasn't fired yet (it is in the future).
            // -pal
            let cur_alarm = self.mux.alarm.get_alarm();
            let now = self.mux.alarm.now();
            let expiration = reference.wrapping_add(dt);
            if !cur_alarm.within_range(reference, expiration) {
                let next = self.mux.next_tick_vals.get();
                if next.is_none_or(|(next_reference, next_dt)| {
                    now.within_range(next_reference, next_reference.wrapping_add(next_dt))
                }) {
                    self.mux.set_alarm(reference, dt);
                }
            } else {
                // current alarm will fire earlier, keep it
            }
        }
    }

    fn get_alarm(&self) -> Self::Ticks {
        let dt_reference = self.dt_reference.get();
        let extension = if dt_reference.extended {
            Self::Ticks::half_max_value()
        } else {
            Self::Ticks::from(0)
        };
        dt_reference.reference_plus_dt().wrapping_add(extension)
    }

    fn minimum_dt(&self) -> Self::Ticks {
        self.mux.alarm.minimum_dt()
    }
}

impl<'a, A: Alarm<'a>> time::AlarmClient for VirtualMuxAlarm<'a, A> {
    fn alarm(&self) {
        self.client.map(|client| client.alarm());
    }
}

/// Structure to control a set of virtual alarms multiplexed together on top of a single alarm.
pub struct MuxAlarm<'a, A: Alarm<'a>> {
    /// Head of the linked list of virtual alarms multiplexed together.
    virtual_alarms: List<'a, VirtualMuxAlarm<'a, A>>,
    /// Number of virtual alarms that are currently enabled.
    enabled: Cell<usize>,
    /// Underlying alarm, over which the virtual alarms are multiplexed.
    alarm: &'a A,
    /// Whether we are firing; used to delay restarted alarms
    firing: Cell<bool>,
    /// Reference to next alarm
    next_tick_vals: Cell<Option<(A::Ticks, A::Ticks)>>,
}

impl<'a, A: Alarm<'a>> MuxAlarm<'a, A> {
    pub const fn new(alarm: &'a A) -> MuxAlarm<'a, A> {
        MuxAlarm {
            virtual_alarms: List::new(),
            enabled: Cell::new(0),
            alarm,
            firing: Cell::new(false),
            next_tick_vals: Cell::new(None),
        }
    }

    pub fn set_alarm(&self, reference: A::Ticks, dt: A::Ticks) {
        self.next_tick_vals.set(Some((reference, dt)));
        self.alarm.set_alarm(reference, dt);
    }

    pub fn disarm(&self) {
        self.next_tick_vals.set(None);
        let _ = self.alarm.disarm();
    }
}

impl<'a, A: Alarm<'a>> time::AlarmClient for MuxAlarm<'a, A> {
    /// When the underlying alarm has fired, we have to multiplex this event back to the virtual
    /// alarms that should now fire.
    fn alarm(&self) {
        // Check whether to fire each alarm. At this level, alarms are one-shot,
        // so a repeating client will set it again in the alarm() callback.
        self.firing.set(true);
        self.virtual_alarms
            .iter()
            .filter(|cur| {
                let dt_ref = cur.dt_reference.get();
                // It is very important to get the current now time as the reference could have been
                // set from now in the previous for_each iteration. We rely on the reference always
                // being in the past when compared to now.
                let now = self.alarm.now();
                cur.armed.get() && !now.within_range(dt_ref.reference, dt_ref.reference_plus_dt())
            })
            .for_each(|cur| {
                let dt_ref = cur.dt_reference.get();
                if dt_ref.extended {
                    // The first part of the extended alarm just fired, leave alarm armed with
                    // remaining time.
                    cur.dt_reference.set(TickDtReference {
                        reference: dt_ref.reference_plus_dt(),
                        dt: A::Ticks::half_max_value(),
                        extended: false,
                    });
                } else {
                    // Alarm fully expired, disarm and fire callback
                    cur.armed.set(false);
                    self.enabled.set(self.enabled.get() - 1);
                    //debug!("  Virtualizer: {:?} outside {:?}-{:?}, fire!", now, cur.reference.get(), cur.reference.get().wrapping_add(cur.dt.get()));
                    cur.alarm();
                }
            });
        self.firing.set(false);
        // Find the soonest alarm client (if any) and set the "next" underlying
        // alarm based on it.  This needs to happen after firing all expired
        // alarms since those may have reset new alarms.
        let now = self.alarm.now();
        let next = self
            .virtual_alarms
            .iter()
            .filter(|cur| cur.armed.get())
            .min_by_key(|cur| {
                let when = cur.dt_reference.get();
                // If the alarm has already expired, then it should be
                // considered as the earliest possible (0 ticks), so it
                // will trigger as soon as possible. This can happen
                // if the alarm expired *after* it was examined in the
                // above loop.
                if !now.within_range(when.reference, when.reference_plus_dt()) {
                    A::Ticks::from(0u32)
                } else {
                    when.reference_plus_dt().wrapping_sub(now)
                }
            });

        // Set the alarm.
        if let Some(valrm) = next {
            let dt_reference = valrm.dt_reference.get();
            self.set_alarm(dt_reference.reference, dt_reference.dt);
        } else {
            self.disarm();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::*;

    struct FakeAlarm<'a> {
        now: Cell<Ticks32>,
        reference: Cell<Ticks32>,
        dt: Cell<Ticks32>,
        armed: Cell<bool>,
        client: OptionalCell<&'a dyn AlarmClient>,
    }

    impl FakeAlarm<'_> {
        fn new() -> Self {
            Self {
                now: Cell::new(1_000u32.into()),
                reference: Cell::new(0u32.into()),
                dt: Cell::new(0u32.into()),
                armed: Cell::new(false),
                client: OptionalCell::empty(),
            }
        }

        /// The emulated delay from when hardware timer to when kernel loop will
        /// run to check if alarms have fired or not.
        pub fn hardware_delay(&self) -> Ticks32 {
            Ticks32::from(10)
        }

        /// Fast forwards time to the next time we would fire an alarm and call client. Returns if
        /// alarm is still armed after triggering client
        pub fn trigger_next_alarm(&self) -> bool {
            if !self.is_armed() {
                return false;
            }
            self.now.set(
                self.reference
                    .get()
                    .wrapping_add(self.dt.get())
                    .wrapping_add(self.hardware_delay()),
            );
            self.client.map(|c| c.alarm());
            self.is_armed()
        }

        /// Runs for the specified number of ticks as long as there are alarms armed.
        pub fn run_for_ticks(&self, left: Ticks32) {
            let final_now = self.now.get().wrapping_add(left);
            let mut left = left.into_u32();

            while self.is_armed() {
                // Ensure that we have enough remaining ticks to handle the next alarm. Reference is
                // always in the past, so we need to figure out the difference between the reference
                // and now to discount the DT the alarm needs to wait by.
                let ticks_from_reference = self.now.get().wrapping_sub(self.reference.get());
                let dt = self
                    .dt
                    .get()
                    .into_u32()
                    .saturating_sub(ticks_from_reference.into_u32());
                if dt <= left {
                    left -= dt;
                    self.trigger_next_alarm();
                } else {
                    break;
                }
            }
            // Ensure that we ate up all of the time we were suppose to run for
            self.now.set(final_now);
        }
    }

    impl Time for FakeAlarm<'_> {
        type Ticks = Ticks32;
        type Frequency = Freq1KHz;

        fn now(&self) -> Ticks32 {
            // Every time we get now, it needs to increment to represent a free running timer
            let new_now = Ticks32::from(self.now.get().into_u32() + 1);
            self.now.set(new_now);
            new_now
        }
    }

    impl<'a> Alarm<'a> for FakeAlarm<'a> {
        fn set_alarm_client(&self, client: &'a dyn AlarmClient) {
            self.client.set(client);
        }

        fn set_alarm(&self, reference: Self::Ticks, dt: Self::Ticks) {
            self.reference.set(reference);
            self.dt.set(dt);
            self.armed.set(true);
        }

        fn get_alarm(&self) -> Self::Ticks {
            self.reference.get().wrapping_add(self.dt.get())
        }

        fn disarm(&self) -> Result<(), ErrorCode> {
            self.armed.set(false);
            Ok(())
        }

        fn is_armed(&self) -> bool {
            self.armed.get()
        }

        fn minimum_dt(&self) -> Self::Ticks {
            0u32.into()
        }
    }

    struct ClientCounter(Cell<usize>);
    impl ClientCounter {
        fn new() -> Self {
            Self(Cell::new(0))
        }
        fn count(&self) -> usize {
            self.0.get()
        }
    }
    impl AlarmClient for ClientCounter {
        fn alarm(&self) {
            self.0.set(self.0.get() + 1);
        }
    }

    fn run_until_disarmed(alarm: &FakeAlarm) {
        // Don't loop forever if we never disarm
        for _ in 0..20 {
            if !alarm.trigger_next_alarm() {
                return;
            }
        }
    }

    #[test]
    fn test_single_max_ticks_dt() {
        let alarm = FakeAlarm::new();
        let client = ClientCounter::new();
        let dt = u32::MAX.into();

        let mux = MuxAlarm::new(&alarm);
        alarm.set_alarm_client(&mux);

        let valarm = VirtualMuxAlarm::new(&mux);
        valarm.setup();
        valarm.set_alarm_client(&client);
        valarm.set_alarm(valarm.now(), dt);

        run_until_disarmed(&alarm);

        assert_eq!(client.count(), 1);
    }

    #[test]
    fn test_multiple_max_ticks_dt() {
        let alarm = FakeAlarm::new();
        let client = ClientCounter::new();
        let dt = u32::MAX.into();

        let mux = MuxAlarm::new(&alarm);
        alarm.set_alarm_client(&mux);

        let v_alarms = &[
            VirtualMuxAlarm::new(&mux),
            VirtualMuxAlarm::new(&mux),
            VirtualMuxAlarm::new(&mux),
        ];

        for (i, v) in v_alarms.iter().enumerate() {
            v.setup();
            v.set_alarm_client(&client);
            // Start with reference in the past since fake alarm now start with 1000 as now()
            v.set_alarm((i as u32).into(), dt);
        }
        run_until_disarmed(&alarm);

        assert_eq!(client.count(), 3);
    }

    struct SetAlarmClient<'a> {
        alarm: &'a VirtualMuxAlarm<'a, FakeAlarm<'a>>,
        dt: u32,
    }

    impl<'a> SetAlarmClient<'a> {
        fn new(alarm: &'a VirtualMuxAlarm<'a, FakeAlarm<'a>>, dt: u32) -> Self {
            Self { alarm, dt }
        }
    }

    impl AlarmClient for SetAlarmClient<'_> {
        fn alarm(&self) {
            self.alarm.set_alarm(self.alarm.now(), self.dt.into());
        }
    }

    #[test]
    fn test_second_alarm_set_during_first_alarm_firing() {
        let alarm = FakeAlarm::new();
        let mux = MuxAlarm::new(&alarm);
        alarm.set_alarm_client(&mux);

        // It is important that 0 is setup last so it is first in the linked list
        let v_alarms = &[VirtualMuxAlarm::new(&mux), VirtualMuxAlarm::new(&mux)];
        v_alarms[1].setup();
        v_alarms[0].setup();

        let set_v1_alarm = SetAlarmClient::new(&v_alarms[1], 100);
        v_alarms[0].set_alarm_client(&set_v1_alarm);

        let counter = ClientCounter::new();
        v_alarms[1].set_alarm_client(&counter);

        // Set the first alarm for 10 ticks in the future. This should then set the second alarm,
        // but not call fired for the second alarm until the timer gets to 100
        v_alarms[0].set_alarm(0.into(), 10.into());
        let still_armed = alarm.trigger_next_alarm();

        // Second alarm should not have triggered yet
        assert!(alarm.now().into_u32() < 100);
        assert_eq!(counter.count(), 0);
        assert!(still_armed);

        let still_armed = alarm.trigger_next_alarm();

        assert!(alarm.now().into_u32() > 100);
        assert_eq!(counter.count(), 1);
        assert!(!still_armed);
    }

    #[test]
    fn test_quick_alarms_not_skipped() {
        let alarm = FakeAlarm::new();
        let client = ClientCounter::new();

        let mux = MuxAlarm::new(&alarm);
        alarm.set_alarm_client(&mux);

        let v_alarms = &[
            VirtualMuxAlarm::new(&mux),
            VirtualMuxAlarm::new(&mux),
            VirtualMuxAlarm::new(&mux),
            VirtualMuxAlarm::new(&mux),
            VirtualMuxAlarm::new(&mux),
            VirtualMuxAlarm::new(&mux),
        ];

        // Precalculated the now and dt for all alarms. The DT should be large enough that the
        // initial check for firing is not true, but after evaluating all alarms, they would all
        // be firing. This happens since time "progresses" every time now() is called, which
        // emulates the clock progressing in real time.
        let now = alarm.now();
        let dt = alarm
            .hardware_delay()
            .wrapping_add(Ticks32::from(v_alarms.len() as u32));

        for v in v_alarms {
            v.setup();
            v.set_alarm_client(&client);
            v.set_alarm(now, dt);
        }

        // Set one alarm to trigger immediately (at the hardware delay) and the other alarm to
        // trigger in the future by some large degree
        v_alarms[0].set_alarm(now, 0.into());
        v_alarms[1].set_alarm(now, 1_000.into());

        // Run the alarm long enough for every alarm but the longer alarm to fire, and all other
        // alarms should have fired once
        alarm.run_for_ticks(Ticks32::from(750));
        assert_eq!(client.count(), v_alarms.len() - 1);
        // Run the alarm long enough for the longer alarm to fire as well and verify count
        alarm.run_for_ticks(Ticks32::from(750));
        assert_eq!(client.count(), v_alarms.len());
    }
}
