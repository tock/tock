use core::cell::Cell;
use hil::alarm::{Alarm, AlarmClient};
use common::{List, ListLink, ListNode};

pub struct VirtualMuxAlarm<'a, Alrm: Alarm + 'a> {
    alarm: &'a MuxAlarm<'a, Alrm>,
    when: Cell<u32>,
    armed: Cell<bool>,
    next: ListLink<'a, VirtualMuxAlarm<'a, Alrm>>,
    client: Cell<Option<&'a AlarmClient>>
}

impl<'a, A: Alarm + 'a> ListNode<'a, VirtualMuxAlarm<'a, A>> for VirtualMuxAlarm<'a, A> {
    fn next(&self) -> &'a ListLink<VirtualMuxAlarm<'a, A>> {
        &self.next
    }
}

impl<'a, Alrm: Alarm> VirtualMuxAlarm<'a, Alrm> {
    pub fn new(mux_alarm: &'a MuxAlarm<'a, Alrm>) -> VirtualMuxAlarm<'a, Alrm> {
        VirtualMuxAlarm {
            alarm: mux_alarm,
            when: Cell::new(0),
            armed: Cell::new(false),
            next: ListLink::empty(),
            client: Cell::new(None)
        }
    }

    pub fn set_client(&'a self, client: &'a AlarmClient) {
        self.alarm.virtual_alarms.push_head(self);
        self.when.set(0);
        self.armed.set(false);
        self.client.set(Some(client));
    }
}

impl<'a, Alrm: Alarm> Alarm for VirtualMuxAlarm<'a, Alrm> {

    type Frequency = Alrm::Frequency;

    fn now(&self) -> u32 {
        self.alarm.alarm.now()
    }

    fn set_alarm(&self, when: u32) {
        let enabled = self.alarm.enabled.get();
        self.alarm.enabled.set(enabled + 1);

        // If there are no other virtual alarms enabled, set the underlying
        // alarm
        if enabled == 0 {
            self.alarm.prev.set(self.alarm.alarm.now());
            self.alarm.alarm.set_alarm(when);
        }
        self.armed.set(true);
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

        let enabled = self.alarm.enabled.get() - 1;
        self.alarm.enabled.set(enabled);

        // If there are not more enabled alarms, disable the underlying alarm
        // completely.
        if enabled == 0 {
            self.alarm.alarm.disable_alarm();
        }
    }

    fn is_armed(&self) -> bool {
        self.armed.get()
    }
}

pub struct MuxAlarm<'a, Alrm: Alarm + 'a> {
    virtual_alarms: List<'a, VirtualMuxAlarm<'a, Alrm>>,
    enabled: Cell<usize>,
    prev: Cell<u32>,
    alarm: &'a Alrm
}

impl<'a, Alrm: Alarm> MuxAlarm<'a, Alrm> {
    pub const fn new(alarm: &'a Alrm) -> MuxAlarm<'a, Alrm> {
        MuxAlarm {
            virtual_alarms: List::new(),
            enabled: Cell::new(0),
            prev: Cell::new(0),
            alarm: alarm
        }
    }
}

impl <'a, Alrm: Alarm> AlarmClient for VirtualMuxAlarm<'a, Alrm> {
    fn fired(&self) {
        self.client.get().map(|client| client.fired() );
    }
}

fn past_from_base(cur: u32, now: u32, prev: u32) -> bool {
    cur.wrapping_sub(now) <= cur.wrapping_sub(prev)
}

impl <'a, Alrm: Alarm> AlarmClient for MuxAlarm<'a, Alrm> {
    fn fired(&self) {

        /*
        // Event Overhead, Timer, Middle Capsule
        // set P3 as low to end test
        unsafe {
        asm! ("\
            movw r3, 0x1058    \n\
            movt r3, 0x400E    \n\
            movs r4, 0x1000    \n\
            str  r4, [r3]      \n\
            "
            :               /* output */
            :               /* input */
            : "r3", "r4"    /* clobbers */
            );
        }
        */

        // Disable the alarm. If there are remaining armed alarms at the end we
        // will enable the alarm again via `set_alarm`
        self.alarm.disable_alarm();

        let now = self.alarm.now();
        let mut next = None;
        let mut min_distance : u32 = u32::max_value();

        for cur in self.virtual_alarms.iter() {
            let should_fire = past_from_base(cur.when.get(),
                                         now, self.prev.get());
            if cur.armed.get() && should_fire {
                cur.armed.set(false);
                self.enabled.set(self.enabled.get() - 1);
                cur.fired();
            } else {
                let distance = cur.when.get().wrapping_sub(now);
                if cur.armed.get() && distance < min_distance {
                    min_distance = distance;
                    next = Some(cur);
                }
            }
        }

        self.prev.set(now);
        next.map(|valrm| self.alarm.set_alarm(valrm.when.get()));
    }
}

