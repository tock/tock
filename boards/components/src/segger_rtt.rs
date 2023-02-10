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

use core::mem::MaybeUninit;
use core_capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use extra_capsules::segger_rtt::{SeggerRtt, SeggerRttMemory};
use kernel::component::Component;
use kernel::hil::time::{self, Alarm};

// Setup static space for the objects.
#[macro_export]
macro_rules! segger_rtt_memory_component_static {
    () => {{
        let rtt_memory = kernel::static_buf!(extra_capsules::segger_rtt::SeggerRttMemory);
        let up_buffer =
            kernel::static_buf!([u8; extra_capsules::segger_rtt::DEFAULT_UP_BUFFER_LENGTH]);
        let down_buffer =
            kernel::static_buf!([u8; extra_capsules::segger_rtt::DEFAULT_DOWN_BUFFER_LENGTH]);

        (rtt_memory, up_buffer, down_buffer)
    };};
}

#[macro_export]
macro_rules! segger_rtt_component_static {
    ($A:ty $(,)?) => {{
        let alarm = kernel::static_buf!(core_capsules::virtual_alarm::VirtualMuxAlarm<'static, $A>);
        let rtt = kernel::static_buf!(
            extra_capsules::segger_rtt::SeggerRtt<
                'static,
                core_capsules::virtual_alarm::VirtualMuxAlarm<'static, $A>,
            >
        );

        (alarm, rtt)
    };};
}

pub struct SeggerRttMemoryRefs<'a> {
    rtt_memory: &'a mut SeggerRttMemory<'a>,
    up_buffer: &'a mut [u8],
    down_buffer: &'a mut [u8],
}

impl<'a> SeggerRttMemoryRefs<'a> {
    pub unsafe fn get_rtt_memory_ptr(&mut self) -> *mut SeggerRttMemory<'a> {
        self.rtt_memory as *mut _
    }
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
        &'static mut MaybeUninit<[u8; extra_capsules::segger_rtt::DEFAULT_UP_BUFFER_LENGTH]>,
        &'static mut MaybeUninit<[u8; extra_capsules::segger_rtt::DEFAULT_DOWN_BUFFER_LENGTH]>,
    );
    type Output = SeggerRttMemoryRefs<'static>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let name = b"Terminal\0";
        let up_buffer_name = name;
        let down_buffer_name = name;
        let up_buffer =
            s.1.write([0; extra_capsules::segger_rtt::DEFAULT_UP_BUFFER_LENGTH]);
        let down_buffer =
            s.2.write([0; extra_capsules::segger_rtt::DEFAULT_DOWN_BUFFER_LENGTH]);

        let rtt_memory = s.0.write(SeggerRttMemory::new_raw(
            up_buffer_name,
            up_buffer.as_ptr(),
            up_buffer.len(),
            down_buffer_name,
            down_buffer.as_ptr(),
            down_buffer.len(),
        ));
        SeggerRttMemoryRefs {
            rtt_memory,
            up_buffer,
            down_buffer,
        }
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
    type Output =
        &'static extra_capsules::segger_rtt::SeggerRtt<'static, VirtualMuxAlarm<'static, A>>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let virtual_alarm_rtt = static_buffer.0.write(VirtualMuxAlarm::new(self.mux_alarm));
        virtual_alarm_rtt.setup();

        // RTT communication channel
        let rtt = static_buffer.1.write(SeggerRtt::new(
            virtual_alarm_rtt,
            self.rtt_memory_refs.rtt_memory,
            self.rtt_memory_refs.up_buffer,
            self.rtt_memory_refs.down_buffer,
        ));

        virtual_alarm_rtt.set_alarm_client(rtt);

        rtt
    }
}
