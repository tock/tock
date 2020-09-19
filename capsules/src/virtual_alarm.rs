//! Virtualize the Alarm interface to enable multiple users of an underlying
//! alarm hardware peripheral.

use core::cell::Cell;
use kernel::common::cells::OptionalCell;
use kernel::common::{List, ListLink, ListNode};
use kernel::hil::time::{self, Alarm, Ticks, Time};
use kernel::ReturnCode;

/// An object to multiplex multiple "virtual" alarms over a single underlying alarm. A
/// `VirtualMuxAlarm` is a node in a linked list of alarms that share the same underlying alarm.
pub struct VirtualMuxAlarm<'a, A: Alarm<'a>> {
    /// Underlying alarm which multiplexes all these virtual alarm.
    mux: &'a MuxAlarm<'a, A>,
    /// Reference time point when this alarm was setup.
    reference: Cell<A::Ticks>,
    /// Duration of this alarm w.r.t. the reference time point. In other words, this alarm should
    /// fire at `reference + dt`.
    dt: Cell<A::Ticks>,
    /// Whether this alarm is currently armed, i.e. whether it should fire when the time has
    /// elapsed.
    armed: Cell<bool>,
    /// Next alarm in the list.
    next: ListLink<'a, VirtualMuxAlarm<'a, A>>,
    /// Alarm client for this node in the list.
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
        self.reference.set(reference);
        self.dt.set(dt);
	
        if !self.armed.get() {
            self.mux.enabled.set(enabled + 1);
            self.armed.set(true);
        }

        // First alarm, so set it
        if enabled == 0 {
	    //debug!("virtual_alarm: first alarm: set it.");
            self.mux.set_alarm(self.reference.get(), self.dt.get());
        } else if self.mux.firing.get() == false {
            // If firing is true, the mux will scan all the alarms after
            // firing and pick the soonest one so do not need to modify the
            // mux. Otherwise, this is an alarm
            // started in a separate code path (e.g., another event).
	    // This new alarm fires sooner if two things are both true:
	    //    1. The current earliest alarm expiration doesn't fall
	    //    in the range of [reference, reference+dt): this means
	    //    it is either in the past (before reference) or the future
	    //    (reference + dt), AND
	    //    2. now falls in the [reference, reference+dt)
	    //    window of the current earliest alarm. This means the
	    //    current earliest alarm hasn't fired yet (it is in the future).
            // -pal
            let cur_alarm = self.mux.alarm.get_alarm();
	    let now = self.mux.alarm.now();
	    let expiration = reference.wrapping_add(dt);
            if !cur_alarm.within_range(reference, expiration) {
		let next = self.mux.next_tick_vals.get();
		if next.map_or(true, |(next_reference, next_dt)| {
		    now.within_range(next_reference,
				     next_reference.wrapping_add(next_dt))
		}) {
                    self.mux.set_alarm(reference, dt);
		}
            } else {
                // current alarm will fire earlier, keep it
            }
        }
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

/// Structure to control a set of virtual alarms multiplexed together on top of a single alarm.
pub struct MuxAlarm<'a, A: Alarm<'a>> {
    /// Head of the linked list of virtual alarms multiplexed together.
    virtual_alarms: List<'a, VirtualMuxAlarm<'a, A>>,
    /// Number of virtual alarms that are currently enabled.
    enabled: Cell<usize>,
    /// Underlying alarm, over which the virtual alarms are multiplexed.
    alarm: &'a A,
    /// Whether we are firing; used to delay restarted alarms
    firing: Cell<bool>,
    /// Reference to next alarm
    next_tick_vals: Cell<Option<(A::Ticks, A::Ticks)>>
}

impl<'a, A: Alarm<'a>> MuxAlarm<'a, A> {
    pub const fn new(alarm: &'a A) -> MuxAlarm<'a, A> {
        MuxAlarm {
            virtual_alarms: List::new(),
            enabled: Cell::new(0),
            alarm: alarm,
            firing: Cell::new(false),
	    next_tick_vals: Cell::new(None),
        }
    }

    pub fn set_alarm(&self, reference: A::Ticks, dt: A::Ticks) {
	self.next_tick_vals.set(Some((reference, dt)));
	self.alarm.set_alarm(reference, dt);
    }

    pub fn disarm(&self) {
	self.next_tick_vals.set(None);
	self.alarm.disarm();
    }
    
}

impl<'a, A: Alarm<'a>> time::AlarmClient for MuxAlarm<'a, A> {
    /// When the underlying alarm has fired, we have to multiplex this event back to the virtual
    /// alarms that should now fire.
    fn alarm(&self) {
        let now = self.alarm.now();
        // Check whether to fire each alarm. At this level, alarms are one-shot,
        // so a repeating client will set it again in the alarm() callback.
        self.firing.set(true);
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
                //debug!("  Virtualizer: {:?} outside {:?}-{:?}, fire!", now, cur.reference.get(), cur.reference.get().wrapping_add(cur.dt.get()));
                cur.alarm();
            });
        self.firing.set(false);
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
            });

        // Set the alarm.
        if let Some(valrm) = next {
            self.set_alarm(valrm.reference.get(), valrm.dt.get());
        } else {
            self.disarm();
        }
    }
}
