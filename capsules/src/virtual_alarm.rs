//! Virtualize the Alarm interface to enable multiple users of an underlying
//! alarm hardware peripheral.

use core::cell::Cell;
use kernel::common::cells::OptionalCell;
use kernel::common::{List, ListLink, ListNode};
use kernel::hil::time::{self, Alarm, Time};

pub struct VirtualMuxAlarm<'a, A: Alarm<'a>> {
    mux: &'a MuxAlarm<'a, A>,
    when: Cell<u32>,
    armed: Cell<bool>,
    next: ListLink<'a, VirtualMuxAlarm<'a, A>>,
    client: OptionalCell<&'a dyn time::AlarmClient>,
    /// Stores a known time in the near past; used for comparisons of time. This
    /// effectively marks the time of what is considered past and future.
    prev: Cell<u32>,
}

impl<'a, A: Alarm<'a>> ListNode<'a, VirtualMuxAlarm<'a, A>> for VirtualMuxAlarm<'a, A> {
    fn next(&self) -> &'a ListLink<VirtualMuxAlarm<'a, A>> {
        &self.next
    }
}

impl<'a, A: Alarm<'a>> VirtualMuxAlarm<'a, A> {
    pub fn new(mux_alarm: &'a MuxAlarm<'a, A>) -> VirtualMuxAlarm<'a, A> {
        VirtualMuxAlarm {
            mux: mux_alarm,
            when: Cell::new(0),
            armed: Cell::new(false),
            next: ListLink::empty(),
            client: OptionalCell::empty(),
            prev: Cell::new(0),
        }
    }
}

impl<'a, A: Alarm<'a>> Time for VirtualMuxAlarm<'a, A> {
    type Frequency = A::Frequency;

    fn max_tics(&self) -> u32 {
        self.mux.alarm.max_tics()
    }

    fn now(&self) -> u32 {
        let now = self.mux.alarm.now();
        self.prev.set(now);
        now
    }
}

impl<'a, A: Alarm<'a>> Alarm<'a> for VirtualMuxAlarm<'a, A> {
    fn set_client(&'a self, client: &'a dyn time::AlarmClient) {
        self.mux.virtual_alarms.push_head(self);
        self.when.set(0);
        self.armed.set(false);
        self.client.set(client);
    }

    fn disable(&self) {
        if !self.armed.get() {
            return;
        }

        self.armed.set(false);

        let enabled = self.mux.enabled.get() - 1;
        self.mux.enabled.set(enabled);

        // If there are not more enabled alarms, disable the underlying alarm
        // completely.
        if enabled == 0 {
            self.mux.alarm.disable();
        }
    }

    fn is_enabled(&self) -> bool {
        self.armed.get()
    }

    fn set_alarm(&self, when: u32) {
        let enabled = self.mux.enabled.get();

        if !self.armed.get() {
            self.mux.enabled.set(enabled + 1);
            self.armed.set(true);
        }

        self.when.set(when);

        if enabled > 0 {
            let cur_alarm = self.mux.alarm.get_alarm();
            let prev = self.mux.prev.get();

            // If this virtual alarm is sooner than the current alarm, reset the
            // underlying alarm
            if cur_alarm.wrapping_sub(prev) > when.wrapping_sub(prev) {
                self.mux.alarm.set_alarm(when);
            }
        } else {
            // Since we are just now enabling the alarm, the underlying mux
            // previous might be really far in the past. Update it with our more
            // recent prev. Our prev was most likely just  updated when the user
            // call now() to calculate the time to use for set_alarm
            self.mux.prev.set(self.prev.get());
            self.mux.alarm.set_alarm(when);
        }
    }

    fn get_alarm(&self) -> u32 {
        self.when.get()
    }
}

impl<'a, A: Alarm<'a>> time::AlarmClient for VirtualMuxAlarm<'a, A> {
    fn fired(&self) {
        self.client.map(|client| client.fired());
    }
}

// MuxAlarm

pub struct MuxAlarm<'a, A: Alarm<'a>> {
    virtual_alarms: List<'a, VirtualMuxAlarm<'a, A>>,
    enabled: Cell<usize>,
    prev: Cell<u32>,
    alarm: &'a A,
}

impl<'a, A: Alarm<'a>> MuxAlarm<'a, A> {
    pub const fn new(alarm: &'a A) -> MuxAlarm<'a, A> {
        MuxAlarm {
            virtual_alarms: List::new(),
            enabled: Cell::new(0),
            prev: Cell::new(0),
            alarm: alarm,
        }
    }
}

fn has_expired(alarm: u32, now: u32, prev: u32) -> bool {
    now.wrapping_sub(prev) >= alarm.wrapping_sub(prev)
}

impl<'a, A: Alarm<'a>> time::AlarmClient for MuxAlarm<'a, A> {
    fn fired(&self) {
        let now = self.alarm.now();

        // Capture this before the loop because it can change while checking
        // each alarm. If a timer fires, it can immediately set a new timer
        // by calling `VirtualMuxAlarm.set_alarm()` which can change `self.prev`
        let prev = self.prev.get();

        // Check whether to fire each alarm. At this level, alarms are one-shot,
        // so a repeating client will set it again in the fired() callback.
        self.virtual_alarms
            .iter()
            .filter(|cur| cur.armed.get() && has_expired(cur.when.get(), now, prev))
            .for_each(|cur| {
                cur.armed.set(false);
                self.enabled.set(self.enabled.get() - 1);
                cur.fired();
            });

        // Find the soonest alarm client (if any) and set the "next" underlying
        // alarm based on it.  This needs to happen after firing all expired
        // alarms since those may have reset new alarms.
        let next = self
            .virtual_alarms
            .iter()
            .filter(|cur| cur.armed.get())
            .min_by_key(|cur| cur.when.get().wrapping_sub(prev));

        self.prev.set(now);
        // If there is an alarm to fire, set the underlying alarm to it
        if let Some(valrm) = next {
            // If we already expired, just call this function recursively until
            // we have serviced  all pending virtual alarms, otherwise set the
            // underlying alarm for the next nearest  value in the future.
            if has_expired(valrm.when.get(), self.alarm.now(), prev) {
                self.fired();
            } else {
                // Trust that the underlying alarm will honor a time in the very
                // recent past (e.g. the last second) will actually fire this
                // callback again now (or very soon).
                self.alarm.set_alarm(valrm.when.get());
            }
        } else {
            self.alarm.disable();
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
