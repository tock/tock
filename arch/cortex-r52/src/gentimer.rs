// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Scheduler timer driver for the Cortex-R52 based on the ARMv8 Generic Timer.
//!
//! This uses the EL1 physical timer (`El1PhysicalTimer`) provided by the
//! `aarch32-cpu` crate to implement Tock's Alarm interface.

// From my understanding: EL0 timer is the same as EL1, Virtual is the
// same than Physical with just an offset, and EL2 is not accessible,
// so only 1 timer is available on R52.

// Initial code was for the scheduler, and dualtimer for Alarm due to a
// misunderstanding, but the  other way around is a bit simplier...

use aarch32_cpu::generic_timer::{El1PhysicalTimer, GenericTimer};

use kernel::hil::time::{Alarm, AlarmClient, Ticks64, Time};
use an536_inst;
use crate::support::isb;

pub struct GenTimer<'a> {
    timer: core::cell::RefCell<El1PhysicalTimer>,
    client: kernel::utilities::cells::OptionalCell<&'a dyn AlarmClient>,
}

impl GenTimer<'_> {
    /// Create a scheduler timer backed by the EL1 physical generic timer.
    ///
    /// Only one handle to the EL1 physical generic timer may be created at a
    /// time; see [`El1PhysicalTimer::new`].
    pub fn new() -> Self {
        let timer = core::cell::RefCell::new(unsafe { El1PhysicalTimer::new() });
        Self {
            timer,
            client: kernel::utilities::cells::OptionalCell::empty(),
        }
    }

    pub fn now64(&self) -> u64 {
        // As per ARM DDI 0487 G6.2.1
        isb();
        let cnt = self.timer.borrow().counter();
        isb();
        cnt
    }

    pub fn set_alarm_u64(&self, cnt: u64) {
        let mut timer = self.timer.borrow_mut();
        timer.counter_compare_set(cnt);
        timer.interrupt_mask(false);
        timer.enable(true);
    }

    pub fn get_alarm_u64(&self) -> u64 {
        self.timer.borrow().counter_compare()
    }

    pub fn handle_interrupt(&self) {
        // Clear the IRQ in HW by canceling condition cmp < cnt
        self.timer.borrow_mut().counter_compare_set(u64::MAX);
        self.client.map(|client| client.alarm());
    }

    pub fn has_fired(&self) -> bool {
        self.timer.borrow().interrupt_status()
    }
}

impl Time for GenTimer<'_> {
    type Frequency = an536_inst::FreqAclk;
    type Ticks = Ticks64;

    fn now(&self) -> Self::Ticks {
        Self::Ticks::from(self.now64())
    }
}

impl<'a> Alarm<'a> for GenTimer<'a> {
    fn set_alarm_client(&self, client: &'a dyn AlarmClient) {
        self.client.set(client);
    }

    fn set_alarm(&self, rf: Self::Ticks, dt: Self::Ticks) {
        self.set_alarm_u64(rf.into_u64() + dt.into_u64());
    }

    fn get_alarm(&self) -> Self::Ticks {
        self.get_alarm_u64().into()
    }

    fn disarm(&self) -> Result<(), kernel::ErrorCode> {
        self.timer.borrow_mut().enable(false);
        //sh::s!("%");
        Ok(())
    }

    fn is_armed(&self) -> bool {
        self.timer.borrow().enabled()
    }

    fn minimum_dt(&self) -> Self::Ticks {
        Self::Ticks::from(16_u64)
    }
}

/*
// Cannot be used twice and Alarm more important
impl SchedulerTimer for GenTimer {
    fn start(&self, us: NonZeroU32) {
        let ticks = self.mhertz * (us.get() as u64);
        let mut timer = self.timer.borrow_mut();
        timer.countdown_set(ticks.try_into().unwrap());
        timer.interrupt_mask(false);
        timer.enable(true);
    }

    fn reset(&self) {
        let mut timer = self.timer.borrow_mut();
        timer.enable(false);
        timer.interrupt_mask(true);
        timer.countdown_set(0);
    }

    fn arm(&self) {
        self.timer.borrow_mut().interrupt_mask(false);
    }

    fn disarm(&self) {
        self.timer.borrow_mut().interrupt_mask(true);
    }

    fn get_remaining_us(&self) -> Option<NonZeroU32> {
        let ticks : u64 = self.timer.borrow().countdown() as u64;
        let remaining = ticks / self.mhertz;
        NonZeroU32::new(remaining as u32)
    }
}
*/
