//! Provide multiple Timers on top of a single underlying Alarm.

use crate::virtual_alarm::VirtualMuxAlarm;
use core::cell::Cell;
use core::cmp;
use kernel::common::cells::{NumericCellExt, OptionalCell};
use kernel::common::{List, ListLink, ListNode};
use kernel::debug;
use kernel::hil::time::{self, Alarm, Ticks, Time, Timer};
use kernel::ReturnCode;

#[derive(Copy, Clone, Debug, PartialEq)]
enum Mode {
    Uninserted = 0,
    Disabled = 1,
    OneShot = 2,
    Repeating = 3,
}

pub struct VirtualTimer<'a, A: Alarm<'a>> {
    mux: &'a MuxTimer<'a, A>,
    when: Cell<A::Ticks>,
    interval: Cell<A::Ticks>,
    mode: Cell<Mode>,
    next: ListLink<'a, VirtualTimer<'a, A>>,
    client: OptionalCell<&'a dyn time::TimerClient>,
}

impl<'a, A: Alarm<'a>> ListNode<'a, VirtualTimer<'a, A>> for VirtualTimer<'a, A> {
    fn next(&self) -> &'a ListLink<VirtualTimer<'a, A>> {
        &self.next
    }
}

impl<'a, A: Alarm<'a>> VirtualTimer<'a, A> {
    pub fn new(mux_timer: &'a MuxTimer<'a, A>) -> VirtualTimer<'a, A> {
        let zero = A::ticks_from_seconds(0);
        let v = VirtualTimer {
            mux: mux_timer,
            when: Cell::new(zero),
            interval: Cell::new(zero),
            mode: Cell::new(Mode::Uninserted),
            next: ListLink::empty(),
            client: OptionalCell::empty(),
        };
        v
    }

    // Start a new timer, configuring its mode and adjusting the
    // underlying alarm if needed.
    fn insert_timer(&'a self, interval: A::Ticks, mode: Mode) -> A::Ticks {
        //        debug!("Inserting timer (mode: {:?}), interval: {}", mode, interval.into_u32());

        if self.mode.get() == Mode::Uninserted {
            //debug!("  - First time, insert.");
            self.mux.timers.push_head(&self);
            self.mode.set(Mode::Disabled);
        }

        if self.mode.get() == Mode::Disabled {
            //            debug!("  - Disabled, increment count on mux.");
            self.mux.enabled.increment();
        }
        self.mode.set(mode);

        // We can't fire faster than the minimum dt of the alarm.
        let real_interval: A::Ticks = A::Ticks::from(cmp::max(
            interval.into_u32(),
            self.mux.alarm.minimum_dt().into_u32(),
        ));

        let now = self.mux.alarm.now();
        self.interval.set(real_interval);
        self.when.set(now.wrapping_add(real_interval));
        self.mux.calculate_alarm(now, real_interval);

        real_interval
    }
}

impl<'a, A: Alarm<'a>> Time for VirtualTimer<'a, A> {
    type Frequency = A::Frequency;
    type Ticks = A::Ticks;

    fn now(&self) -> A::Ticks {
        self.mux.alarm.now()
    }
}

impl<'a, A: Alarm<'a>> Timer<'a> for VirtualTimer<'a, A> {
    fn set_timer_client(&'a self, client: &'a dyn time::TimerClient) {
        self.client.set(client);
    }

    fn cancel(&self) -> ReturnCode {
        match self.mode.get() {
            Mode::Uninserted | Mode::Disabled => ReturnCode::SUCCESS,
            Mode::OneShot | Mode::Repeating => {
                self.mode.set(Mode::Disabled);
                self.mux.enabled.decrement();

                // If there are not more enabled timers, disable the
                // underlying alarm.
                if self.mux.enabled.get() == 0 {
                    self.mux.alarm.disarm();
                }
                ReturnCode::SUCCESS
            }
        }
    }

    fn interval(&self) -> Option<Self::Ticks> {
        match self.mode.get() {
            Mode::Uninserted | Mode::Disabled => None,
            Mode::OneShot | Mode::Repeating => Some(self.interval.get()),
        }
    }

    fn is_oneshot(&self) -> bool {
        self.mode.get() == Mode::OneShot
    }

    fn is_repeating(&self) -> bool {
        self.mode.get() == Mode::Repeating
    }

    fn is_enabled(&self) -> bool {
        match self.mode.get() {
            Mode::Uninserted => false,
            Mode::Disabled => false,
            Mode::OneShot => true,
            Mode::Repeating => true,
        }
    }

    fn oneshot(&'a self, interval: Self::Ticks) -> Self::Ticks {
        self.insert_timer(interval, Mode::OneShot)
    }

    fn repeating(&'a self, interval: Self::Ticks) -> Self::Ticks {
        self.insert_timer(interval, Mode::Repeating)
    }

    fn time_remaining(&self) -> Option<Self::Ticks> {
        match self.mode.get() {
            Mode::Uninserted | Mode::Disabled => None,
            Mode::OneShot | Mode::Repeating => {
                let when = self.when.get();
                let now = self.mux.alarm.now();
                Some(when.wrapping_sub(now))
            }
        }
    }
}

impl<'a, A: Alarm<'a>> time::AlarmClient for VirtualTimer<'a, A> {
    fn alarm(&self) {
        match self.mode.get() {
            Mode::Uninserted | Mode::Disabled => {} // Do nothing
            Mode::OneShot => {
                self.mode.set(Mode::Disabled);
                self.client.map(|client| client.timer());
            }
            Mode::Repeating => {
                // By setting the 'now' to be 'when', this ensures
                // the the repeating timer fires at a fixed interval:
                // it'll fire at when + (k * interval), for k=0...n.
                let when = self.when.get();
                let interval = self.interval.get();
                self.when.set(when.wrapping_add(interval));
                self.mux.calculate_alarm(when, interval);
                self.client.map(|client| client.timer());
            }
        }
    }
}

pub struct MuxTimer<'a, A: Alarm<'a>> {
    timers: List<'a, VirtualTimer<'a, A>>,
    enabled: Cell<usize>,
    alarm: &'a VirtualMuxAlarm<'a, A>,
}

impl<'a, A: Alarm<'a>> MuxTimer<'a, A> {
    pub const fn new(alarm: &'a VirtualMuxAlarm<'a, A>) -> MuxTimer<'a, A> {
        MuxTimer {
            timers: List::new(),
            enabled: Cell::new(0),
            alarm: alarm,
        }
    }

    fn calculate_alarm(&'a self, now: A::Ticks, interval: A::Ticks) {
        if self.enabled.get() == 1 {
            //debug!("Calculating alarm: first, so set it.");
            self.alarm.set_alarm(now, interval);
        } else {
            // If the current alarm doesn't fall within the range of
            // [now, now + interval), this means this new alarm
            // will fire sooner. This covers the case when the current
            // alarm is in the past, because it must have already fired
            // and the bottom half is pending. -pal
            let cur_alarm = self.alarm.get_alarm();
            let when = now.wrapping_add(interval);
            if !cur_alarm.within_range(now, when) {
                //debug!("Calculating alarm: earlier, so set it.");
                self.alarm.set_alarm(now, interval);
            } else {
                //debug!("Calculating alarm: later, do nothing.");
                // current alarm will fire earlier, keep it
            }
        }
    }
}

impl<'a, A: Alarm<'a>> time::AlarmClient for MuxTimer<'a, A> {
    fn alarm(&self) {
        // The "now" is when the alarm fired, not the current
        // time; this is case there was some delay. This also
        // ensures that all other timers are >= now.
        let now = self.alarm.get_alarm();
        //debug!("Alarm virtualizer: alarm called at {}", now.into_u32());
        // Check whether to fire each timer. At this level, alarms are one-shot,
        // so a repeating timer will reset its `when` in the alarm() callback.
        self.timers
            .iter()
            .filter(|cur| {
                cur.is_enabled()
                    && !now.within_range(
                        cur.when.get().wrapping_sub(cur.interval.get()),
                        cur.when.get(),
                    )
            })
            .for_each(|cur| {
                //debug!("  Virtualizer: {} outside {}-{}, fire!", now.into_u32(), cur.reference.get().into_u32(), cur.reference.get().wrapping_add(cur.dt.get()).into_u32());
                cur.alarm();
            });

        // Find the soonest alarm client (if any) and set the "next" underlying
        // alarm based on it.  This needs to happen after firing all expired
        // alarms since those may have reset new alarms.
        let next = self
            .timers
            .iter()
            .filter(|cur| cur.is_enabled())
            .min_by_key(|cur| cur.when.get().wrapping_sub(now).into_u32());

        // Set the alarm.
        if let Some(valrm) = next {
            self.alarm
                .set_alarm(now, valrm.when.get().wrapping_sub(now));
        } else {
            self.alarm.disarm();
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
