//! Virtualize the Alarm interface to enable multiple users of an underlying
//! alarm hardware peripheral.

use core::cell::Cell;
use kernel::common::{List, ListLink, ListNode};
use kernel::hil::time::{self, Alarm, Time};

pub struct VirtualMuxAlarm<'a, Alrm: Alarm + 'a> {
    mux: &'a MuxAlarm<'a, Alrm>,
    when: Cell<u32>,
    armed: Cell<bool>,
    next: ListLink<'a, VirtualMuxAlarm<'a, Alrm>>,
    client: Cell<Option<&'a time::Client>>,
}

impl<'a, A: Alarm> ListNode<'a, VirtualMuxAlarm<'a, A>> for VirtualMuxAlarm<'a, A> {
    fn next(&self) -> &'a ListLink<VirtualMuxAlarm<'a, A>> {
        &self.next
    }
}

impl<'a, Alrm: Alarm> VirtualMuxAlarm<'a, Alrm> {
    pub fn new(mux_alarm: &'a MuxAlarm<'a, Alrm>) -> VirtualMuxAlarm<'a, Alrm> {
        VirtualMuxAlarm {
            mux: mux_alarm,
            when: Cell::new(0),
            armed: Cell::new(false),
            next: ListLink::empty(),
            client: Cell::new(None),
        }
    }

    pub fn set_client(&'a self, client: &'a time::Client) {
        self.mux.virtual_alarms.push_head(self);
        self.when.set(0);
        self.armed.set(false);
        self.client.set(Some(client));
    }
}

impl<'a, Alrm: Alarm> Time for VirtualMuxAlarm<'a, Alrm> {
    type Frequency = Alrm::Frequency;

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

    fn is_armed(&self) -> bool {
        self.armed.get()
    }
}

impl<'a, Alrm: Alarm> Alarm for VirtualMuxAlarm<'a, Alrm> {
    fn now(&self) -> u32 {
        self.mux.alarm.now()
    }

    fn set_alarm(&self, when: u32) {
        let enabled = self.mux.enabled.get();

        if !self.is_armed() {
            self.mux.enabled.set(enabled + 1);
            self.armed.set(true);
        }

        if enabled > 0 {
            let cur_alarm = self.mux.alarm.get_alarm();
            let now = self.now();

            if cur_alarm.wrapping_sub(now) > when.wrapping_sub(now) {
                self.mux.prev.set(self.mux.alarm.now());
                self.mux.alarm.set_alarm(when);
            }
        } else {
            self.mux.prev.set(self.mux.alarm.now());
            self.mux.alarm.set_alarm(when);
        }

        self.when.set(when);
    }

    fn get_alarm(&self) -> u32 {
        self.when.get()
    }
}

impl<'a, Alrm: Alarm> time::Client for VirtualMuxAlarm<'a, Alrm> {
    fn fired(&self) {
        self.client.get().map(|client| client.fired());
    }
}

// MuxAlarm

pub struct MuxAlarm<'a, Alrm: Alarm + 'a> {
    virtual_alarms: List<'a, VirtualMuxAlarm<'a, Alrm>>,
    enabled: Cell<usize>,
    prev: Cell<u32>,
    alarm: &'a Alrm,
}

impl<'a, Alrm: Alarm> MuxAlarm<'a, Alrm> {
    pub const fn new(alarm: &'a Alrm) -> MuxAlarm<'a, Alrm> {
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

impl<'a, Alrm: Alarm> time::Client for MuxAlarm<'a, Alrm> {
    fn fired(&self) {
        let now = self.alarm.now();

        // Capture this before the loop because it can change while checking
        // each alarm. If a timer fires, it can immediately set a new timer
        // by calling `VirtualMuxAlarm.set_alarm()` which can change `self.prev`
        // to the current timer time.
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
        let next = self.virtual_alarms
            .iter()
            .filter(|cur| cur.armed.get())
            .min_by_key(|cur| cur.when.get().wrapping_sub(now));

        self.prev.set(now);
        // If there is an alarm to fire, set the underlying alarm to it
        if let Some(valrm) = next {
            self.alarm.set_alarm(valrm.when.get());
            if has_expired(valrm.when.get(), self.alarm.now(), prev) {
                self.fired();
            }
        } else {
            self.alarm.disable();
        }
    }
}
