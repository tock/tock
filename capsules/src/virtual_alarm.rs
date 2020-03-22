//! Virtualize the Alarm interface to enable multiple users of an underlying
//! alarm hardware peripheral.

use core::cell::Cell;
use kernel::common::cells::OptionalCell;
use kernel::common::{List, ListLink, ListNode};
use kernel::debug;
use kernel::hil::time::AlarmClient;
use kernel::hil::time::{self, Alarm, Time};

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

        self.mux.alarm.disable();
        // checks whether the mux has been notified of the new alarm
        // if yes, than the mux is alreafy in a reschedule function
        // is not, it notifies it
        if !self.mux.notified.get() {
            self.mux.reschedule();
        }
    }

    fn is_enabled(&self) -> bool {
        self.armed.get()
    }

    fn set_alarm(&self, tics: u32) {
        if !self.armed.get() {
            self.armed.set(true);
        }

        self.when.set(tics);
        self.mux.alarm.disable();
        // checks whether the mux has been notified of the new alarm
        // if yes, than the mux is alreafy in a reschedule function
        // is not, it notifies it
        if !self.mux.notified.get() {
            self.mux.reschedule();
        }
    }

    fn get_alarm(&self) -> u32 {
        self.when.get()
    }
}

impl<A: Alarm<'a>> time::AlarmClient for VirtualMuxAlarm<'a, A> {
    fn fired(&self) {
        self.client.map(|client| client.fired());
    }

    fn update(&self, tics: usize) {
        self.client.map(|client| client.update(tics));
    }
}

// MuxAlarm

pub struct MuxAlarm<'a, A: Alarm<'a>> {
    virtual_alarms: List<'a, VirtualMuxAlarm<'a, A>>,
    alarm: &'a A,
    notified: Cell<bool>,
}

impl<A: Alarm<'a>> MuxAlarm<'a, A> {
    pub const fn new(alarm: &'a A) -> MuxAlarm<'a, A> {
        MuxAlarm {
            virtual_alarms: List::new(),
            alarm: alarm,
            notified: Cell::new(false),
        }
    }

    pub fn disable(&self) {
        self.alarm.disable();
    }

    pub fn reschedule(&self) {
        // the mux has been notified
        self.notified.set(true);
        // if we are in the fired handler, reschedule will be called
        // from update
        // debug!("reschedule");
        // disable timer so that is does not fire while we are
        // in this function
        self.alarm.disable();
        let tics = self.alarm.get_alarm() as usize;
        let next = self
            .virtual_alarms
            .iter()
            .map(|cur| {
                cur.update(tics);
                cur
            })
            .filter(|cur| cur.armed.get())
            .min_by_key(|cur| cur.when.get());

        if let Some(valarm) = next {
            // debug!("reschedule alarm to {}", actual_tics);
            self.alarm.set_alarm(valarm.when.get());
        } else {
            self.alarm.disable();
            self.notified.set(false);
        }
    }
}

impl<A: Alarm<'a>> time::AlarmClient for MuxAlarm<'a, A> {
    fn fired(&self) {
        // get the number of tics that fired the alarm

        // iterate every virtual alarm in the mux and inform it
        // that a number of tics have expirted and get the
        // next minimum tics to set
        self.reschedule();
        // let now = self.alarm.now();

        // // Capture this before the loop because it can change while checking
        // // each alarm. If a timer fires, it can immediately set a new timer
        // // by calling `VirtualMuxAlarm.set_alarm()` which can change `self.prev`
        // // to the current timer time.
        // let prev = self.prev.get();

        // // Check whether to fire each alarm. At this level, alarms are one-shot,
        // // so a repeating client will set it again in the fired() callback.
        // self.virtual_alarms
        //     .iter()
        //     .filter(|cur| cur.armed.get() && has_expired(cur.when.get(), now, prev))
        //     .for_each(|cur| {
        //         cur.armed.set(false);
        //         self.enabled.set(self.enabled.get() - 1);
        //         cur.fired();
        //     });

        // // Find the soonest alarm client (if any) and set the "next" underlying
        // // alarm based on it.  This needs to happen after firing all expired
        // // alarms since those may have reset new alarms.
        // let next = self
        //     .virtual_alarms
        //     .iter()
        //     .filter(|cur| cur.armed.get())
        //     .min_by_key(|cur| cur.when.get().wrapping_sub(now));

        // self.prev.set(now);
        // // If there is an alarm to fire, set the underlying alarm to it
        // if let Some(valrm) = next {
        //     self.alarm.set_alarm(valrm.when.get());
        //     if has_expired(valrm.when.get(), self.alarm.now(), prev) {
        //         self.fired();
        //     }
        // } else {
        //     self.alarm.disable();
        // }
    }
}
