
use common::{List, ListLink, ListNode};
use core::cell::Cell;
use hil::alarm::{Alarm, AlarmClient};

pub struct VirtualMuxAlarm<'a, Alrm: Alarm + 'a> {
    mux: &'a MuxAlarm<'a, Alrm>,
    when: Cell<u32>,
    armed: Cell<bool>,
    next: ListLink<'a, VirtualMuxAlarm<'a, Alrm>>,
    client: Cell<Option<&'a AlarmClient>>,
}

impl<'a, A: Alarm + 'a> ListNode<'a, VirtualMuxAlarm<'a, A>> for VirtualMuxAlarm<'a, A> {
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

    pub fn set_client(&'a self, client: &'a AlarmClient) {
        self.mux.virtual_alarms.push_head(self);
        self.when.set(0);
        self.armed.set(false);
        self.client.set(Some(client));
    }
}

impl<'a, Alrm: Alarm> Alarm for VirtualMuxAlarm<'a, Alrm> {
    type Frequency = Alrm::Frequency;

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

    fn disable_alarm(&self) {
        if !self.armed.get() {
            return;
        }

        self.armed.set(false);

        let enabled = self.mux.enabled.get() - 1;
        self.mux.enabled.set(enabled);

        // If there are not more enabled alarms, disable the underlying alarm
        // completely.
        if enabled == 0 {
            self.mux.alarm.disable_alarm();
        }
    }

    fn is_armed(&self) -> bool {
        self.armed.get()
    }
}

impl<'a, Alrm: Alarm> AlarmClient for VirtualMuxAlarm<'a, Alrm> {
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

fn past_from_base(cur: u32, now: u32, prev: u32) -> bool {
    now.wrapping_sub(prev) >= cur.wrapping_sub(prev)
}

impl<'a, Alrm: Alarm> AlarmClient for MuxAlarm<'a, Alrm> {
    fn fired(&self) {
        // Disable the alarm. If there are remaining armed alarms at the end we
        // will enable the alarm again via `set_alarm`
        self.alarm.disable_alarm();

        let now = self.alarm.now();

        // Check whether to fire each alarm. At this level, alarms are one-shot,
        // so a repeating client will set it again in the fired() callback.
        for cur in self.virtual_alarms.iter() {
            let should_fire = past_from_base(cur.when.get(), now + 100, self.prev.get());
            if cur.armed.get() && should_fire {
                cur.armed.set(false);
                self.enabled.set(self.enabled.get() - 1);
                cur.fired();
            }
        }

        // Find the soonest alarm client (if any) and set the "next" underlying
        // alarm based on it.
        let mut next = None;
        let mut min_distance: u32 = u32::max_value();
        for cur in self.virtual_alarms.iter() {
            if cur.armed.get() {
                let distance = cur.when.get().wrapping_sub(now);
                if cur.armed.get() && distance < min_distance {
                    min_distance = distance;
                    next = Some(cur);
                }
            }
        }

        self.prev.set(now);
        // If there is an alarm to fire, set the underlying alarm to it
        next.map(|valrm| self.alarm.set_alarm(valrm.when.get()));
    }
}
