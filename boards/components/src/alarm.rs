//! Components for hardware timer Alarms.
//!
//! This provides two components, `AlarmMuxComponent`, which provides a
//! multiplexed interface to a hardware alarm, and `AlarmDriverComponent`,
//! which provides an alarm system call interface.
//!
//! Usage
//! -----
//! ```rust
//! let ast = &sam4l::ast::AST;
//! let mux_alarm = components::alarm::AlarmMuxComponent::new(ast)
//!     .finalize(components::alarm_mux_component_helper!(sam4l::ast::Ast));
//! ast.configure(mux_alarm);
//! let alarm = components::alarm::AlarmDriverComponent::new(board_kernel, mux_alarm)
//!     .finalize(components::alarm_component_helper!(sam4l::ast::Ast));
//! ```

// Author: Philip Levis <pal@cs.stanford.edu>
// Last modified: 12/21/2019

use core::mem::MaybeUninit;

use capsules::alarm::AlarmDriver;
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::time::{self, Alarm};
use kernel::static_init_half;

// Setup static space for the objects.
#[macro_export]
macro_rules! alarm_mux_component_helper {
    ($A:ty $(,)?) => {{
        use capsules::virtual_alarm::MuxAlarm;
        use core::mem::MaybeUninit;
        static mut BUF: MaybeUninit<MuxAlarm<'static, $A>> = MaybeUninit::uninit();
        &mut BUF
    };};
}

// Setup static space for the objects.
#[macro_export]
macro_rules! alarm_component_helper {
    ($A:ty $(,)?) => {{
        use capsules::alarm::AlarmDriver;
        use capsules::virtual_alarm::VirtualMuxAlarm;
        use core::mem::MaybeUninit;
        static mut BUF1: MaybeUninit<VirtualMuxAlarm<'static, $A>> = MaybeUninit::uninit();
        static mut BUF2: MaybeUninit<AlarmDriver<'static, VirtualMuxAlarm<'static, $A>>> =
            MaybeUninit::uninit();
        (&mut BUF1, &mut BUF2)
    };};
}

pub struct AlarmMuxComponent<A: 'static + time::Alarm<'static>> {
    alarm: &'static A,
}

impl<A: 'static + time::Alarm<'static>> AlarmMuxComponent<A> {
    pub fn new(alarm: &'static A) -> AlarmMuxComponent<A> {
        AlarmMuxComponent { alarm }
    }
}

impl<A: 'static + time::Alarm<'static>> Component for AlarmMuxComponent<A> {
    type StaticInput = &'static mut MaybeUninit<MuxAlarm<'static, A>>;
    type Output = &'static MuxAlarm<'static, A>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let mux_alarm = static_init_half!(
            static_buffer,
            MuxAlarm<'static, A>,
            MuxAlarm::new(self.alarm)
        );

        self.alarm.set_alarm_client(mux_alarm);
        mux_alarm
    }
}

pub struct AlarmDriverComponent<A: 'static + time::Alarm<'static>> {
    board_kernel: &'static kernel::Kernel,
    driver_num: u32,
    alarm_mux: &'static MuxAlarm<'static, A>,
}

impl<A: 'static + time::Alarm<'static>> AlarmDriverComponent<A> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: u32,
        mux: &'static MuxAlarm<'static, A>,
    ) -> AlarmDriverComponent<A> {
        AlarmDriverComponent {
            board_kernel: board_kernel,
            driver_num: driver_num,
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

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let virtual_alarm1 = static_init_half!(
            static_buffer.0,
            VirtualMuxAlarm<'static, A>,
            VirtualMuxAlarm::new(self.alarm_mux)
        );
        let alarm = static_init_half!(
            static_buffer.1,
            AlarmDriver<'static, VirtualMuxAlarm<'static, A>>,
            AlarmDriver::new(
                virtual_alarm1,
                self.board_kernel.create_grant(self.driver_num, &grant_cap)
            )
        );

        virtual_alarm1.set_alarm_client(alarm);
        alarm
    }
}
