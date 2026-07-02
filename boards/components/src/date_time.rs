// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Component for Date and Time initialisation.
//!
//! Authors: Irina Bradu <irinabradu.a@gmail.com>
//!          Remus Rughinis <remus.rughinis.007@gmail.com>
//!
//! Usage
//! -----
//!
//! '''rust
//!     let date_time = components::date_time::DateTimeComponent::new(
//!         board_kernel,
//!         capsules_extra::date_time::DRIVER_NUM,
//!         &peripherals.rtc,
//!     )
//!     .finalize(rtc_component_static!(stm32f429zi::rtc::Rtc<'static>));
//! '''

use core::mem::MaybeUninit;

use capsules_extra::date_time::DateTimeCapsule;
use kernel::capabilities::MemoryAllocationCapability;
use kernel::component::Component;
use kernel::hil::date_time;

#[macro_export]
macro_rules! date_time_component_static {
    ($R:ty $(,)?) => {{
        let rtc = kernel::static_buf!(capsules_extra::date_time::DateTimeCapsule<'static, $R>);
        (rtc)
    };};
}

pub struct DateTimeComponent<
    D: 'static + date_time::DateTime<'static>,
    CAP: MemoryAllocationCapability + 'static,
> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    rtc: &'static D,
    mem_cap: CAP,
}

impl<D: 'static + date_time::DateTime<'static>, CAP: MemoryAllocationCapability + 'static>
    DateTimeComponent<D, CAP>
{
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        rtc: &'static D,
        mem_cap: CAP,
    ) -> DateTimeComponent<D, CAP> {
        DateTimeComponent {
            board_kernel,
            driver_num,
            rtc,
            mem_cap,
        }
    }
}

impl<
        D: 'static + date_time::DateTime<'static> + kernel::deferred_call::DeferredCallClient,
        CAP: MemoryAllocationCapability + 'static,
    > Component for DateTimeComponent<D, CAP>
{
    type StaticInput = &'static mut MaybeUninit<DateTimeCapsule<'static, D>>;

    type Output = &'static DateTimeCapsule<'static, D>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let grant_date_time = self
            .board_kernel
            .create_grant(self.driver_num, &self.mem_cap);

        let date_time = s.write(DateTimeCapsule::new(self.rtc, grant_date_time));
        date_time::DateTime::set_client(self.rtc, date_time);
        date_time
    }
}
