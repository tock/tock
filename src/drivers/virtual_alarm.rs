use core::cell::Cell;
use hil::alarm::{Alarm, AlarmClient};

pub struct VirtualMuxAlarm<'a, Alrm: Alarm + 'a> {
    alarm: &'a MuxAlarm<'a, Alrm>,
    when: Cell<u32>,
    armed: Cell<bool>,
    next: Cell<Option<&'a VirtualMuxAlarm<'a, Alrm>>>,
    client: Cell<Option<&'a AlarmClient>>
}

impl<'a, Alrm: Alarm> VirtualMuxAlarm<'a, Alrm> {
    pub fn new(mux_alarm: &'a MuxAlarm<'a, Alrm>) -> VirtualMuxAlarm<'a, Alrm> {
        VirtualMuxAlarm {
            alarm: mux_alarm,
            when: Cell::new(0),
            armed: Cell::new(false),
            next: Cell::new(None),
            client: Cell::new(None)
        }
    }

    pub fn set_client(&'a self, client: &'a AlarmClient) {
        self.next.set(self.alarm.virtual_alarms.get());
        self.alarm.virtual_alarms.set(Some(self));
        self.when.set(0);
        self.armed.set(false);
        self.client.set(Some(client));
    }
}

impl<'a, Alrm: Alarm> Alarm for VirtualMuxAlarm<'a, Alrm> {
    fn now(&self) -> u32 {
        self.alarm.alarm.now()
    }

    fn set_alarm(&self, when: u32) {
        let enabled = self.alarm.enabled.get();
        self.alarm.enabled.set(enabled + 1);
        if enabled == 0 {
            self.alarm.alarm.set_alarm(when);
        }
        self.armed.set(true);
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
        if enabled == 0 {
            self.alarm.alarm.disable_alarm();
        }
    }
}

pub struct MuxAlarm<'a, Alrm: Alarm + 'a> {
    virtual_alarms: Cell<Option<&'a VirtualMuxAlarm<'a, Alrm>>>,
    enabled: Cell<usize>,
    prev: Cell<u32>,
    alarm: &'a Alrm
}

impl<'a, Alrm: Alarm> MuxAlarm<'a, Alrm> {
    pub const fn new(alarm: &'a Alrm) -> MuxAlarm<'a, Alrm> {
        MuxAlarm {
            virtual_alarms: Cell::new(None),
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

fn in_between(cur: u32, now: u32, prev: u32) -> bool {
    if now >= prev {
        cur <= now && cur >= prev
    } else {
        cur <= prev && cur >= now
    }
}

impl <'a, Alrm: Alarm> AlarmClient for MuxAlarm<'a, Alrm> {
    fn fired(&self) {
        let now = self.alarm.now();
        let mut next = None;
        let mut min_distance : u32 = u32::max_value();
        // We know at least one of the virtual_alarms is armed
        let mut ocur = self.virtual_alarms.get();
        loop {
            match ocur {
                None => break,
                Some(cur) => {
                    let should_fire = in_between(cur.when.get(),
                                                 now, self.prev.get());
                    if cur.armed.get() && should_fire {
                        cur.armed.set(false);
                        cur.fired();
                    } else {
                        let distance = cur.when.get().wrapping_sub(now);
                        if cur.armed.get() && distance < min_distance {
                            min_distance = distance;
                            next = Some(cur);
                        }
                    }
                    ocur = cur.next.get();
                }
            }
        }
        self.prev.set(now);
        match next {
            None => {
                self.alarm.disable_alarm();
            },
            Some(valrm) => {
                self.alarm.set_alarm(valrm.when.get());
            }
        }
    }
}

