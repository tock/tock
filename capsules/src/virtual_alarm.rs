//! Virtualize the Alarm interface to enable multiple users of an underlying
//! alarm hardware peripheral.

use core::cell::Cell;
use kernel::common::cells::OptionalCell;
use kernel::common::{List, ListLink, ListNode};
use kernel::hil::time::{Alarm, AlarmClient, Ticks, Time};

pub struct VirtualMuxAlarm<'a, A: Alarm<'a>> {
    mux: &'a MuxAlarm<'a, A>,
    when: Cell<A::Ticks>,
    armed: Cell<bool>,
    next: ListLink<'a, VirtualMuxAlarm<'a, A>>,
    client: OptionalCell<&'a dyn AlarmClient>,
}

impl<A: Alarm<'a>> ListNode<'a, VirtualMuxAlarm<'a, A>> for VirtualMuxAlarm<'a, A> {
    fn next(&self) -> &'a ListLink<VirtualMuxAlarm<'a, A>> {
        &self.next
    }
}

impl<A: Alarm<'a>> VirtualMuxAlarm<'a, A> {
    pub fn new(mux_alarm: &'a MuxAlarm<'a, A>) -> VirtualMuxAlarm<'a, A> {
        VirtualMuxAlarm {
            mux: mux_alarm,
            when: Cell::new(A::Ticks::from(0)),
            armed: Cell::new(false),
            next: ListLink::empty(),
            client: OptionalCell::empty(),
        }
    }
}

impl<A: Alarm<'a>> Time for VirtualMuxAlarm<'a, A> {
    type Ticks = A::Ticks;
    type Frequency = A::Frequency;

    fn now(&self) -> Self::Ticks {
        self.mux.alarm.now()
    }
}

impl<A: Alarm<'a>> Alarm<'a> for VirtualMuxAlarm<'a, A> {
    fn set_client(&'a self, client: &'a dyn AlarmClient) {
        self.mux.virtual_alarms.push_head(self);
        self.when.set(A::Ticks::from(0));
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

    fn set_alarm(&self, when: Self::Ticks) {
        let enabled = self.mux.enabled.get();

        if !self.armed.get() {
            self.mux.enabled.set(enabled + 1);
            self.armed.set(true);
        }

        if enabled > 0 {
            let cur_alarm = self.mux.alarm.get_alarm();
            let now = self.now();

            // `when` is before `cur_alarm`
            if !A::Ticks::expired(now, when, cur_alarm) {
                self.mux.prev.set(self.mux.alarm.now());
                self.mux.alarm.set_alarm(when);
            }
        } else {
            self.mux.prev.set(self.mux.alarm.now());
            self.mux.alarm.set_alarm(when);
        }

        self.when.set(when);
    }

    fn get_alarm(&self) -> Self::Ticks {
        self.when.get()
    }
}

impl<A: Alarm<'a>> AlarmClient for VirtualMuxAlarm<'a, A> {
    fn fired(&self) {
        self.client.map(|client| client.fired());
    }
}

// MuxAlarm

pub struct MuxAlarm<'a, A: Alarm<'a>> {
    virtual_alarms: List<'a, VirtualMuxAlarm<'a, A>>,
    enabled: Cell<usize>,
    prev: OptionalCell<A::Ticks>,
    alarm: &'a A,
}

impl<A: Alarm<'a>> MuxAlarm<'a, A> {
    pub const fn new(alarm: &'a A) -> MuxAlarm<'a, A> {
        MuxAlarm {
            virtual_alarms: List::new(),
            enabled: Cell::new(0),
            prev: OptionalCell::empty(),
            alarm: alarm,
        }
    }
}

impl<A: Alarm<'a>> AlarmClient for MuxAlarm<'a, A> {
    fn fired(&self) {
        let now = self.alarm.now();

        // Capture this before the loop because it can change while checking
        // each alarm. If a timer fires, it can immediately set a new timer
        // by calling `VirtualMuxAlarm.set_alarm()` which can change `self.prev`
        // to the current timer time.
        let prev = self.prev.unwrap_or(A::Ticks::from(0));

        // Check whether to fire each alarm. At this level, alarms are one-shot,
        // so a repeating client will set it again in the fired() callback.
        self.virtual_alarms
            .iter()
            .filter(|cur| cur.armed.get() && A::Ticks::expired(prev, now, cur.when.get()))
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
            .min_by_key(|cur| cur.when.get().wrapping_sub(now).into_u32());

        self.prev.set(now);
        // If there is an alarm to fire, set the underlying alarm to it
        if let Some(valrm) = next {
            self.alarm.set_alarm(valrm.when.get());
            if A::Ticks::expired(prev, self.alarm.now(), valrm.when.get()) {
                self.fired();
            }
        } else {
            self.alarm.disable();
        }
    }
}
