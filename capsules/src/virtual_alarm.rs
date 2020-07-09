//! Virtualize the Alarm interface to enable multiple users of an underlying
//! alarm hardware peripheral.

use core::cell::Cell;
use kernel::common::cells::OptionalCell;
use kernel::common::{List, ListLink, ListNode};
use kernel::hil::time::{self, Alarm, Ticks, Time};
use kernel::ReturnCode;
//use kernel::debug;

pub struct VirtualMuxAlarm<'a, A: Alarm<'a>> {
    mux: &'a MuxAlarm<'a, A>,
    reference: Cell<A::Ticks>,
    dt: Cell<A::Ticks>,
    armed: Cell<bool>,
    next: ListLink<'a, VirtualMuxAlarm<'a, A>>,
    client: OptionalCell<&'a dyn time::AlarmClient>,
}

impl<'a, A: Alarm<'a>> ListNode<'a, VirtualMuxAlarm<'a, A>> for VirtualMuxAlarm<'a, A> {
    fn next(&self) -> &'a ListLink<VirtualMuxAlarm<'a, A>> {
        &self.next
    }
}

impl<'a, A: Alarm<'a>> VirtualMuxAlarm<'a, A> {
    pub fn new(mux_alarm: &'a MuxAlarm<'a, A>) -> VirtualMuxAlarm<'a, A> {
        let zero = A::ticks_from_seconds(0);
        VirtualMuxAlarm {
            mux: mux_alarm,
            reference: Cell::new(zero),
            dt: Cell::new(zero),
            armed: Cell::new(false),
            next: ListLink::empty(),
            client: OptionalCell::empty(),
        }
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
    fn set_alarm_client(&'a self, client: &'a dyn time::AlarmClient) {
        self.mux.virtual_alarms.push_head(self);
        // Reset the alarm state: should it do this? Does not seem
        // to be semantically correct. What if you just wanted to
        // change the callback. Keeping it but skeptical. -pal
        self.reference.set(A::Ticks::from(0 as u32));
        self.dt.set(A::Ticks::from(0 as u32));
        self.armed.set(false);
        self.client.set(client);
    }

    fn disarm(&self) -> ReturnCode {
        if !self.armed.get() {
            return ReturnCode::SUCCESS;
        }

        self.armed.set(false);

        let enabled = self.mux.enabled.get() - 1;
        self.mux.enabled.set(enabled);

        // If there are not more enabled alarms, disable the underlying alarm
        // completely.
        if enabled == 0 {
            self.mux.alarm.disarm();
        }
        ReturnCode::SUCCESS
    }

    fn is_armed(&self) -> bool {
        self.armed.get()
    }

    fn set_alarm(&self, reference: Self::Ticks, dt: Self::Ticks) {
        let enabled = self.mux.enabled.get();

        if !self.armed.get() {
            self.mux.enabled.set(enabled + 1);
            self.armed.set(true);
        }

        // First alarm, so set it
        if enabled == 0 {
            self.mux.alarm.set_alarm(reference, dt);
        } else {
            // If the current alarm doesn't fall within the range of
            // [reference, reference + dt), this means this new alarm
            // will fire sooner. This covers the case even when the new
            // alarm has already expired. -pal
            let cur_alarm = self.mux.alarm.get_alarm();
            if !cur_alarm.within_range(reference, reference.wrapping_add(dt)) {
                self.mux.alarm.set_alarm(reference, dt);
            } else {
                // current alarm will fire earlier, keep it
            }
        }
        self.reference.set(reference);
        self.dt.set(dt);
    }

    fn get_alarm(&self) -> Self::Ticks {
        self.reference.get().wrapping_add(self.dt.get())
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

// MuxAlarm

pub struct MuxAlarm<'a, A: Alarm<'a>> {
    virtual_alarms: List<'a, VirtualMuxAlarm<'a, A>>,
    enabled: Cell<usize>,
    alarm: &'a A,
}

impl<'a, A: Alarm<'a>> MuxAlarm<'a, A> {
    pub const fn new(alarm: &'a A) -> MuxAlarm<'a, A> {
        MuxAlarm {
            virtual_alarms: List::new(),
            enabled: Cell::new(0),
            alarm: alarm,
        }
    }
}

impl<'a, A: Alarm<'a>> time::AlarmClient for MuxAlarm<'a, A> {
    fn alarm(&self) {
        // The "now" is when the alarm fired, not the current
        // time; this is case there was some delay. This also
        // ensures that all other timers are >= now.
        let now = self.alarm.get_alarm();
        //debug!("Alarm virtualizer: alarm called at {}", now.into_u32());
        // Check whether to fire each alarm. At this level, alarms are one-shot,
        // so a repeating client will set it again in the alarm() callback.
        self.virtual_alarms
            .iter()
            .filter(|cur| {
                cur.armed.get()
                    && !now.within_range(
                        cur.reference.get(),
                        cur.reference.get().wrapping_add(cur.dt.get()),
                    )
            })
            .for_each(|cur| {
                cur.armed.set(false);
                self.enabled.set(self.enabled.get() - 1);
                //debug!("  Virtualizer: {} outside {}-{}, fire!", now.into_u32(), cur.reference.get().into_u32(), cur.reference.get().wrapping_add(cur.dt.get()).into_u32());
                cur.alarm();
            });

        // Find the soonest alarm client (if any) and set the "next" underlying
        // alarm based on it.  This needs to happen after firing all expired
        // alarms since those may have reset new alarms.
        let next = self
            .virtual_alarms
            .iter()
            .filter(|cur| cur.armed.get())
            .min_by_key(|cur| {
                cur.reference
                    .get()
                    .wrapping_add(cur.dt.get())
                    .wrapping_sub(now)
                    .into_u32()
            });

        // Set the alarm.
        if let Some(valrm) = next {
            self.alarm.set_alarm(valrm.reference.get(), valrm.dt.get());
        } else {
            self.alarm.disarm();
        }
    }
}

#[cfg(test)]
mod test {
    use super::has_expired;

    #[test]
    fn has_expired_with_zero_reference() {
        assert_eq!(has_expired(1, 1, 0), true);
        assert_eq!(has_expired(1, 0, 0), false);
        assert_eq!(has_expired(0, 1, 0), true);
    }
}
