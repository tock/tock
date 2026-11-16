// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Scheduler timer driver for the Cortex-R52 based on the ARMv8 Generic Timer.

// From my understanding: EL0 timer is the same as EL1, Virtual is the
// same than Physical with just an offset, and EL2 is not accessible,
// so only 1 timer is available on R52.

// Initial code was for the scheduler, and dualtimer for Alarm due to a
// misunderstanding, but the  other way around is a bit simplier...

use kernel::hil::time::{Alarm, AlarmClient, Ticks64, Time};
use crate::config;
use crate::support::isb;
use crate::reg;

pub struct GenTimer<'a> {
    client: kernel::utilities::cells::OptionalCell<&'a dyn AlarmClient>,
}

impl GenTimer<'_> {
    pub fn new() -> Self {
        Self {
            client: kernel::utilities::cells::OptionalCell::empty(),
        }
    }

    pub fn now64(&self) -> u64 {
        // As per ARM DDI 0487 G6.2.1
        isb();
        let cnt = reg::CNTPCT_get();
        isb();
        cnt
    }

    pub fn set_alarm_u64(&self, cnt: u64) {
        reg::CNTP_CVAL_set(cnt);
        reg::CNTP_CTL_set(reg::CNTP_CTL_ENABLE);
    }

    pub fn get_alarm_u64(&self) -> u64 {
        reg::CNTP_CVAL_get()
    }

    pub fn handle_interrupt(&self) {
        // Clear the IRQ in HW by canceling condition CVAL < CT
        reg::CNTP_CVAL_set(u64::MAX);
        self.client.map(|client| client.alarm());
    }

    pub fn has_fired(&self) -> bool {
        (reg::CNTP_CTL_get() & reg::CNTP_CTL_ISTATUS) != 0
    }
}

impl Time for GenTimer<'_> {
    type Frequency = config::FreqAclk;
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
        reg::CNTP_CTL_set(0);
        Ok(())
    }

    fn is_armed(&self) -> bool {
        (reg::CNTP_CTL_get() & reg::CNTP_CTL_ENABLE) != 0
    }

    fn minimum_dt(&self) -> Self::Ticks {
        // Chosen almost randomly!
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
