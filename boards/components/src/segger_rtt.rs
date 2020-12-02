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
//! let rtt_memory = components::segger_rtt::SeggerRttMemoryComponent::new().finalize(());
//! let rtt = components::segger_rtt::SeggerRttComponent::new(mux_alarm, rtt_memory)
//!     .finalize(components::segger_rtt_component_helper!(nrf52832::rtc::Rtc));
//! ```

// Author: Guillaume Endignoux <guillaumee@google.com>
// Last modified: 07/02/2020

use capsules::segger_rtt::{
    SeggerRtt, SeggerRttMemory, DEFAULT_DOWN_BUFFER_LENGTH, DEFAULT_UP_BUFFER_LENGTH,
};
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::time::{self, Alarm};
use kernel::{static_init, static_init_half};

// Setup static space for the objects.
#[macro_export]
macro_rules! segger_rtt_component_helper {
    ($A:ty $(,)?) => {{
        use capsules::segger_rtt::SeggerRtt;
        use capsules::virtual_alarm::VirtualMuxAlarm;
        use core::mem::MaybeUninit;
        static mut BUF1: MaybeUninit<VirtualMuxAlarm<'static, $A>> = MaybeUninit::uninit();
        static mut BUF2: MaybeUninit<SeggerRtt<'static, VirtualMuxAlarm<'static, $A>>> =
            MaybeUninit::uninit();
        (&mut BUF1, &mut BUF2)
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
    type StaticInput = ();
    type Output = SeggerRttMemoryRefs<'static>;

    unsafe fn finalize(self, _s: Self::StaticInput) -> Self::Output {
        let name = b"Terminal\0";
        let up_buffer_name = name;
        let down_buffer_name = name;
        let up_buffer = static_init!(
            [u8; DEFAULT_UP_BUFFER_LENGTH],
            [0; DEFAULT_UP_BUFFER_LENGTH]
        );
        let down_buffer = static_init!(
            [u8; DEFAULT_DOWN_BUFFER_LENGTH],
            [0; DEFAULT_DOWN_BUFFER_LENGTH]
        );

        let rtt_memory = static_init!(
            SeggerRttMemory,
            SeggerRttMemory::new_raw(
                up_buffer_name,
                up_buffer.as_ptr(),
                up_buffer.len(),
                down_buffer_name,
                down_buffer.as_ptr(),
                down_buffer.len()
            )
        );
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
    type Output = &'static capsules::segger_rtt::SeggerRtt<'static, VirtualMuxAlarm<'static, A>>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let virtual_alarm_rtt = static_init_half!(
            static_buffer.0,
            VirtualMuxAlarm<'static, A>,
            VirtualMuxAlarm::new(self.mux_alarm)
        );

        // RTT communication channel
        let rtt = static_init_half!(
            static_buffer.1,
            SeggerRtt<'static, VirtualMuxAlarm<'static, A>>,
            SeggerRtt::new(
                virtual_alarm_rtt,
                self.rtt_memory_refs.rtt_memory,
                self.rtt_memory_refs.up_buffer,
                self.rtt_memory_refs.down_buffer
            )
        );

        virtual_alarm_rtt.set_alarm_client(rtt);

        rtt
    }
}
