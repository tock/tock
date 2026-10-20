// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Scheduler timer driver for the Cortex-R52 based on the ARMv8 Generic Timer.
//!
//! This uses the EL1 physical timer (`El1PhysicalTimer`) provided by the
//! `aarch32-cpu` crate to implement Tock's Alarm interface.

// Initial code was for the scheduler, and dualtimer for Alarm due to a
// misunderstanding, but the  other way around is a bit simplier...

use aarch32_cpu::generic_timer::{El1PhysicalTimer, GenericTimer};

use kernel::hil::time::{Alarm, AlarmClient, Frequency, Ticks64, Time};

pub struct GenTimer<'a, F: Frequency> {
    timer: core::cell::RefCell<El1PhysicalTimer>,
    client: kernel::utilities::cells::OptionalCell<&'a dyn AlarmClient>,
    _f: core::marker::PhantomData<F>, // Need by F generic
}

impl<'a, F: Frequency> GenTimer<'a, F> {
    /// Create a scheduler timer backed by the EL1 physical generic timer.
    ///
    /// Only one handle to the EL1 physical generic timer may be created at a
    /// time; see [`El1PhysicalTimer::new`].

    // From my understanding: EL0 timer is the same as EL1, Virtual is the
    // same than Physical with just an offset, and EL2 is not accessible,
    // so only 1 timer is available on R52.

    pub unsafe fn new() -> Self {
        let timer = core::cell::RefCell::new(unsafe { El1PhysicalTimer::new() });
        Self {
            timer,
            client: kernel::utilities::cells::OptionalCell::empty(),
            _f: core::marker::PhantomData,
        }
    }

    pub fn now(&self) -> u64 {
        self.timer.borrow().counter()
    }

    pub fn handle_interrupt(&self) {
        // Nothing to do regarding the HW IRQ, as the trigger is edge
        // and anyway to way to clear it in software.
        self.client.map(|client| client.alarm());
    }
}

impl<'a, F: Frequency> Time for GenTimer<'a, F> {
    type Frequency = F;
    type Ticks = Ticks64;

    fn now(&self) -> Self::Ticks {
        let ticks = self.timer.borrow().counter();
        Self::Ticks::from(ticks)
    }
}

impl<'a, F: Frequency> Alarm<'a> for GenTimer<'a, F> {
    fn set_alarm_client(&self, client: &'a dyn AlarmClient) {
        self.client.set(client);
    }

    fn set_alarm(&self, rf: Self::Ticks, dt: Self::Ticks) {
        let mut timer = self.timer.borrow_mut();
        let abs = rf.into_u64() + dt.into_u64();
        timer.counter_compare_set(abs);
        timer.interrupt_mask(false);
        timer.enable(true);
    }

    fn get_alarm(&self) -> Self::Ticks {
        self.timer.borrow().counter_compare().into()
    }

    fn disarm(&self) -> Result<(), kernel::ErrorCode> {
        self.timer.borrow_mut().enable(false);
        Ok(())
    }

    fn is_armed(&self) -> bool {
        self.timer.borrow().enabled()
    }

    fn minimum_dt(&self) -> Self::Ticks {
        Self::Ticks::from(2_u64)
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
