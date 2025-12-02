// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Components for creating a virtual scheduler timer.

use crate::virtual_scheduler_timer::hil::time;
use capsules_core::virtualizers::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use capsules_system::virtual_scheduler_timer::VirtualSchedulerTimer;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil;

// Setup static space for the objects.
#[macro_export]
macro_rules! virtual_scheduler_timer_component_static {
    ($A: ty $(,)?) => {{
        let alarm = kernel::static_buf!(
            capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<'static, $A>
        );
        let scheduler_timer = kernel::static_buf!(
            capsules_system::virtual_scheduler_timer::VirtualSchedulerTimer<
                capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<'static, $A>,
            >
        );

        (alarm, scheduler_timer)
    };};
}

pub type VirtualSchedulerTimerComponentType<A> = VirtualSchedulerTimer<VirtualMuxAlarm<'static, A>>;

pub struct VirtualSchedulerTimerComponent<A: 'static + time::Alarm<'static>> {
    mux_alarm: &'static MuxAlarm<'static, A>,
}

impl<A: 'static + time::Alarm<'static>> VirtualSchedulerTimerComponent<A> {
    pub fn new(mux_alarm: &'static MuxAlarm<'static, A>) -> Self {
        Self { mux_alarm }
    }
}

impl<A: 'static + time::Alarm<'static>> Component for VirtualSchedulerTimerComponent<A> {
    type StaticInput = (
        &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
        &'static mut MaybeUninit<VirtualSchedulerTimer<VirtualMuxAlarm<'static, A>>>,
    );
    type Output = &'static VirtualSchedulerTimer<VirtualMuxAlarm<'static, A>>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let systick_virtual_alarm = static_buffer.0.write(VirtualMuxAlarm::new(self.mux_alarm));
        systick_virtual_alarm.setup();

        let scheduler_timer = static_buffer
            .1
            .write(VirtualSchedulerTimer::new(systick_virtual_alarm));

        scheduler_timer
    }
}

#[macro_export]
macro_rules! virtual_scheduler_timer_no_mux_component_static {
    ($A: ty $(,)?) => {{
        let scheduler_timer = kernel::static_buf!(
            capsules_system::virtual_scheduler_timer::VirtualSchedulerTimer<$A>
        );

        scheduler_timer
    };};
}

pub type VirtualSchedulerTimerNoMuxComponentType<A> = VirtualSchedulerTimer<A>;

pub struct VirtualSchedulerTimerNoMuxComponent<A: 'static + time::Alarm<'static>> {
    alarm: &'static A,
}

impl<A: 'static + time::Alarm<'static>> VirtualSchedulerTimerNoMuxComponent<A> {
    pub fn new(alarm: &'static A) -> Self {
        Self { alarm }
    }
}

impl<A: 'static + time::Alarm<'static>> Component for VirtualSchedulerTimerNoMuxComponent<A> {
    type StaticInput = &'static mut MaybeUninit<VirtualSchedulerTimer<A>>;
    type Output = &'static VirtualSchedulerTimer<A>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let scheduler_timer = static_buffer.write(VirtualSchedulerTimer::new(self.alarm));

        scheduler_timer
    }
}
