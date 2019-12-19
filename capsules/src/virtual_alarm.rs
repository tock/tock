//! Virtualize the Alarm interface to enable multiple users of an underlying
//! alarm hardware peripheral.

use core::cell::Cell;
use kernel::common::cells::OptionalCell;
use kernel::common::{List, ListLink, ListNode};
use kernel::hil::time::{self, Alarm, Time, TimerWrappingOps};

pub struct VirtualMuxAlarm<'a, A: Alarm<'a>> {
    mux: &'a MuxAlarm<'a, A>,
    when: Cell<u32>,
    armed: Cell<bool>,
    next: ListLink<'a, VirtualMuxAlarm<'a, A>>,
    client: OptionalCell<&'a dyn time::AlarmClient>,
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
            when: Cell::new(0),
            armed: Cell::new(false),
            next: ListLink::empty(),
            client: OptionalCell::empty(),
        }
    }
}

impl<A: Alarm<'a>> Time for VirtualMuxAlarm<'a, A> {
    type Frequency = A::Frequency;

    type TimerWrappingOps = A::TimerWrappingOps;

    fn max_tics(&self) -> u32 {
        self.mux.alarm.max_tics()
    }

    fn now(&self) -> u32 {
        self.mux.alarm.now()
    }
}

impl<A: Alarm<'a>> Alarm<'a> for VirtualMuxAlarm<'a, A> {
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

        if enabled > 0 {
            let cur_alarm = self.mux.alarm.get_alarm();
            let now = self.now();

            if <A::TimerWrappingOps>::wrapping_sub(cur_alarm, now)
                > <A::TimerWrappingOps>::wrapping_sub(when, now)
            {
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

impl<A: Alarm<'a>> time::AlarmClient for VirtualMuxAlarm<'a, A> {
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

impl<A: Alarm<'a>> MuxAlarm<'a, A> {
    pub const fn new(alarm: &'a A) -> MuxAlarm<'a, A> {
        MuxAlarm {
            virtual_alarms: List::new(),
            enabled: Cell::new(0),
            prev: Cell::new(0),
            alarm: alarm,
        }
    }
}

fn has_expired<'a, A>(alarm: u32, now: u32, prev: u32) -> bool
where
    A: Alarm<'a>,
{
    <A::TimerWrappingOps>::wrapping_sub(now, prev)
        >= <A::TimerWrappingOps>::wrapping_sub(alarm, prev)
}

impl<A: Alarm<'a>> time::AlarmClient for MuxAlarm<'a, A> {
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
            .filter(|cur| cur.armed.get() && has_expired::<'a, A>(cur.when.get(), now, prev))
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
            .min_by_key(|cur| <A::TimerWrappingOps>::wrapping_sub(cur.when.get(), now));

        self.prev.set(now);
        // If there is an alarm to fire, set the underlying alarm to it
        if let Some(valrm) = next {
            self.alarm.set_alarm(valrm.when.get());
            if has_expired::<'a, A>(valrm.when.get(), self.alarm.now(), prev) {
                self.fired();
            }
        } else {
            self.alarm.disable();
        }
    }
}
