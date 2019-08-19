//! Virtualize the Alarm interface to enable multiple users of an underlying
//! alarm hardware peripheral.

use core::cell::Cell;
use kernel::common::cells::OptionalCell;
use kernel::common::{List, ListLink, ListNode};
use kernel::hil::time::{self, Alarm, Time};
use kernel::ReturnCode;

pub struct VirtualMuxAlarm<'a, Alrm: Alarm<'a>> {
    mux: &'a MuxAlarm<'a, Alrm>,
    when: Cell<u32>,
    armed: Cell<bool>,
    next: ListLink<'a, VirtualMuxAlarm<'a, Alrm>>,
    client: OptionalCell<&'a time::AlarmClient>,
}

impl<A: Alarm<'a>> ListNode<'a, VirtualMuxAlarm<'a, A>> for VirtualMuxAlarm<'a, A> {
    fn next(&self) -> &'a ListLink<VirtualMuxAlarm<'a, A>> {
        &self.next
    }
}

impl<Alrm: Alarm<'a>> VirtualMuxAlarm<'a, Alrm> {
    pub fn new(mux_alarm: &'a MuxAlarm<'a, Alrm>) -> VirtualMuxAlarm<'a, Alrm> {
        VirtualMuxAlarm {
            mux: mux_alarm,
            when: Cell::new(0),
            armed: Cell::new(false),
            next: ListLink::empty(),
            client: OptionalCell::empty(),
        }
    }

    pub fn set_client(&'a self, client: &'a time::AlarmClient) {
        self.mux.virtual_alarms.push_head(self);
        self.when.set(0);
        self.armed.set(false);
        self.client.set(client);
    }
}

impl<Alrm: Alarm<'a>> Time for VirtualMuxAlarm<'a, Alrm> {
    type Frequency = Alrm::Frequency;

    fn max_tics(&self) -> u32 {
        self.mux.alarm.max_tics()
    }

    fn now(&self) -> u32 {
        self.mux.alarm.now()
    }
}

impl<Alrm: Alarm<'a>> Alarm<'a> for VirtualMuxAlarm<'a, Alrm> {
    fn disable(&self) -> ReturnCode {
        if !self.armed.get() {
            return ReturnCode::SUCCESS;
        }

        self.armed.set(false);

        let enabled = self.mux.enabled.get() - 1;
        self.mux.enabled.set(enabled);

        // If there are not more enabled alarms, disable the underlying alarm
        // completely.
        if enabled == 0 {
            self.mux.alarm.disable();
        }

        ReturnCode::SUCCESS
    }

    fn is_enabled(&self) -> bool {
        self.armed.get()
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

impl<Alrm: Alarm<'a>> time::AlarmClient for VirtualMuxAlarm<'a, Alrm> {
    fn fired(&self) {
        self.client.map(|client| client.fired());
    }
}

// MuxAlarm

pub struct MuxAlarm<'a, Alrm: Alarm<'a>> {
    virtual_alarms: List<'a, VirtualMuxAlarm<'a, Alrm>>,
    enabled: Cell<usize>,
    prev: Cell<u32>,
    alarm: &'a Alrm,
}

impl<Alrm: Alarm<'a>> MuxAlarm<'a, Alrm> {
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

impl<Alrm: Alarm<'a>> time::AlarmClient for MuxAlarm<'a, Alrm> {
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
        let next = self
            .virtual_alarms
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
