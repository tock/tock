// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Component for SeggerRttMemory.
//!
//! This provides two `Component`s:
//! - `SeggerRttMemoryComponent`, which creates suitable memory for the Segger
//!   RTT capsule.
//! - `SeggerRttComponent`, which instantiates the Segger RTT capsule.
//!
//! Usage
//! -----
//! ```rust
//! let rtt_memory = components::segger_rtt::SeggerRttMemoryComponent::new()
//!     .finalize(components::segger_rtt_memory_component_static!());
//! let rtt = components::segger_rtt::SeggerRttComponent::new(mux_alarm, rtt_memory)
//!     .finalize(components::segger_rtt_component_static!(nrf52832::rtc::Rtc));
//! ```

// Author: Guillaume Endignoux <guillaumee@google.com>
// Last modified: 07/02/2020

use capsules_core::virtualizers::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::time::{self, Alarm};
use kernel::utilities::cells::VolatileCell;
use segger::rtt::{SeggerRtt, SeggerRttMemory};

// Setup static space for the objects.
#[macro_export]
macro_rules! segger_rtt_memory_component_static {
    () => {{
        let rtt_memory = kernel::static_named_buf!(segger::rtt::SeggerRttMemory, "_SEGGER_RTT");
        let up_buffer = kernel::static_buf!(
            [kernel::utilities::cells::VolatileCell<u8>; segger::rtt::DEFAULT_UP_BUFFER_LENGTH]
        );
        let down_buffer = kernel::static_buf!(
            [kernel::utilities::cells::VolatileCell<u8>; segger::rtt::DEFAULT_DOWN_BUFFER_LENGTH]
        );

        (rtt_memory, up_buffer, down_buffer)
    };};
}

#[macro_export]
macro_rules! segger_rtt_component_static {
    ($A:ty $(,)?) => {{
        let alarm = kernel::static_buf!(
            capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<'static, $A>
        );
        let rtt = kernel::static_buf!(
            segger::rtt::SeggerRtt<
                'static,
                capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<'static, $A>,
            >
        );

        (alarm, rtt)
    };};
}

pub struct SeggerRttMemoryRefs<'a> {
    pub rtt_memory: &'a mut SeggerRttMemory<'a>,
}

pub struct SeggerRttMemoryComponent {}

impl SeggerRttMemoryComponent {
    pub fn new() -> SeggerRttMemoryComponent {
        SeggerRttMemoryComponent {}
    }
}

impl Component for SeggerRttMemoryComponent {
    type StaticInput = (
        &'static mut MaybeUninit<SeggerRttMemory<'static>>,
        &'static mut MaybeUninit<[VolatileCell<u8>; segger::rtt::DEFAULT_UP_BUFFER_LENGTH]>,
        &'static mut MaybeUninit<[VolatileCell<u8>; segger::rtt::DEFAULT_DOWN_BUFFER_LENGTH]>,
    );
    type Output = SeggerRttMemoryRefs<'static>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let name = b"Terminal\0";
        let up_buffer_name = name;
        let down_buffer_name = name;
        let up_buffer =
            s.1.write([const { VolatileCell::new(0) }; segger::rtt::DEFAULT_UP_BUFFER_LENGTH]);
        let down_buffer =
            s.2.write([const { VolatileCell::new(0) }; segger::rtt::DEFAULT_DOWN_BUFFER_LENGTH]);

        let rtt_memory = s.0.write(SeggerRttMemory::new_raw(
            up_buffer_name,
            up_buffer,
            down_buffer_name,
            down_buffer,
        ));
        SeggerRttMemoryRefs { rtt_memory }
    }
}

pub struct SeggerRttComponent<A: 'static + time::Alarm<'static>> {
    mux_alarm: &'static MuxAlarm<'static, A>,
    rtt_memory_refs: SeggerRttMemoryRefs<'static>,
}

impl<A: 'static + time::Alarm<'static>> SeggerRttComponent<A> {
    pub fn new(
        mux_alarm: &'static MuxAlarm<'static, A>,
        rtt_memory_refs: SeggerRttMemoryRefs<'static>,
    ) -> SeggerRttComponent<A> {
        SeggerRttComponent {
            mux_alarm,
            rtt_memory_refs,
        }
    }
}

impl<A: 'static + time::Alarm<'static>> Component for SeggerRttComponent<A> {
    type StaticInput = (
        &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
        &'static mut MaybeUninit<SeggerRtt<'static, VirtualMuxAlarm<'static, A>>>,
    );
    type Output = &'static SeggerRtt<'static, VirtualMuxAlarm<'static, A>>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let virtual_alarm_rtt = static_buffer.0.write(VirtualMuxAlarm::new(self.mux_alarm));
        virtual_alarm_rtt.setup();

        // RTT communication channel
        let rtt = static_buffer.1.write(SeggerRtt::new(
            virtual_alarm_rtt,
            self.rtt_memory_refs.rtt_memory,
        ));

        virtual_alarm_rtt.set_alarm_client(rtt);

        rtt
    }
}
