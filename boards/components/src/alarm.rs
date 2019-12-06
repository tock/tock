//! Component for hardware timer Alarms.
//!
//! This provides one component, `AlarmDriverComponent`, which provides
//! an alarm system call interface.
//!
//! Usage
//! -----
//! ```rust
//! let alarm = components::alarm::AlarmDriverComponent::new(board_kernel, mux_alarm)
//!        .finalize(components::alarm_component_helper!(sam4l::ast::Ast));
//! ```

// Author: Philip Levis <pal@cs.stanford.edu>
// Last modified: 6/20/2018

#![allow(dead_code)] // Components are intended to be conditionally included

use core::mem::MaybeUninit;

use capsules::alarm::AlarmDriver;
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::time;
use kernel::static_init_half;

// Setup static space for the objects.
#[macro_export]
macro_rules! alarm_component_helper {
    ($A:ty) => {{
        use capsules::alarm::AlarmDriver;
        use core::mem::MaybeUninit;
        static mut BUF1: MaybeUninit<VirtualMuxAlarm<'static, $A>> = MaybeUninit::uninit();
        static mut BUF2: MaybeUninit<AlarmDriver<'static, VirtualMuxAlarm<'static, $A>>> =
            MaybeUninit::uninit();
        (&mut BUF1, &mut BUF2)
    };};
}

pub struct AlarmDriverComponent<A: 'static + time::Alarm<'static>> {
    board_kernel: &'static kernel::Kernel,
    alarm_mux: &'static MuxAlarm<'static, A>,
}

impl<A: 'static + time::Alarm<'static>> AlarmDriverComponent<A> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        mux: &'static MuxAlarm<'static, A>,
    ) -> AlarmDriverComponent<A> {
        AlarmDriverComponent {
            board_kernel: board_kernel,
            alarm_mux: mux,
        }
    }
}

impl<A: 'static + time::Alarm<'static>> Component for AlarmDriverComponent<A> {
    type StaticInput = (
        &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
        &'static mut MaybeUninit<AlarmDriver<'static, VirtualMuxAlarm<'static, A>>>,
    );
    type Output = &'static AlarmDriver<'static, VirtualMuxAlarm<'static, A>>;

    unsafe fn finalize(&mut self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let virtual_alarm1 = static_init_half!(
            static_buffer.0,
            VirtualMuxAlarm<'static, A>,
            VirtualMuxAlarm::new(self.alarm_mux)
        );
        let alarm = static_init_half!(
            static_buffer.1,
            AlarmDriver<'static, VirtualMuxAlarm<'static, A>>,
            AlarmDriver::new(virtual_alarm1, self.board_kernel.create_grant(&grant_cap))
        );

        time::Alarm::set_client(virtual_alarm1, alarm);
        alarm
    }
}
