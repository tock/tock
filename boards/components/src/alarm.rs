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
//! let mux_alarm = components::alarm::AlarmMuxComponent::create(
//!     ast,
//!     components::alarm_mux_component_buf!(sam4l::ast::Ast)
//! );
//! ast.configure(mux_alarm);
//! let alarm = components::alarm::AlarmDriverComponent::create(
//!     (board_kernel, mux_alarm),
//!     components::alarm_component_buf!(sam4l::ast::Ast)
//! );
//! ```

// Author: Philip Levis <pal@cs.stanford.edu>
// Last modified: 12/21/2019

use capsules::alarm::AlarmDriver;
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use kernel::component::CreateComponent;
use kernel::hil::time;
use kernel::StaticUninitializedBuffer;
use kernel::{capabilities, create_capability};

// Setup static space for the objects.
#[macro_export]
macro_rules! alarm_mux_component_buf {
    ($A:ty) => {{
        use capsules::virtual_alarm::MuxAlarm;
        $crate::static_buf!(MuxAlarm<'static, $A>)
    };};
}

// Setup static space for the objects.
#[macro_export]
macro_rules! alarm_component_buf {
    ($A:ty) => {{
        use capsules::alarm::AlarmDriver;
        use capsules::virtual_alarm::VirtualMuxAlarm;
        (
            $crate::static_buf!(VirtualMuxAlarm<'static, $A>),
            $crate::static_buf!(AlarmDriver<'static, VirtualMuxAlarm<'static, $A>>),
        )
    };};
}

pub struct AlarmMuxComponent<A: 'static + time::Alarm<'static>> {
    _phantom: core::marker::PhantomData<A>,
}

impl<A: 'static + time::Alarm<'static>> CreateComponent for AlarmMuxComponent<A> {
    type Input = &'static A;
    type StaticInput = StaticUninitializedBuffer<MuxAlarm<'static, A>>;
    type Output = &'static MuxAlarm<'static, A>;

    unsafe fn create(alarm: Self::Input, static_input: Self::StaticInput) -> Self::Output {
        let mux_alarm = static_input.initialize(MuxAlarm::new(alarm));

        time::Alarm::set_client(alarm, mux_alarm);
        mux_alarm
    }
}

pub struct AlarmDriverComponent<A: 'static + time::Alarm<'static>> {
    _phantom: core::marker::PhantomData<A>,
}

impl<A: 'static + time::Alarm<'static>> CreateComponent for AlarmDriverComponent<A> {
    type Input = (&'static kernel::Kernel, &'static MuxAlarm<'static, A>);
    type StaticInput = (
        StaticUninitializedBuffer<VirtualMuxAlarm<'static, A>>,
        StaticUninitializedBuffer<AlarmDriver<'static, VirtualMuxAlarm<'static, A>>>,
    );
    type Output = &'static AlarmDriver<'static, VirtualMuxAlarm<'static, A>>;

    unsafe fn create(input: Self::Input, static_input: Self::StaticInput) -> Self::Output {
        let (board_kernel, alarm_mux) = input;

        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let virtual_alarm1 = static_input.0.initialize(VirtualMuxAlarm::new(alarm_mux));
        let alarm = static_input.1.initialize(AlarmDriver::new(
            virtual_alarm1,
            board_kernel.create_grant(&grant_cap),
        ));

        time::Alarm::set_client(virtual_alarm1, alarm);
        alarm
    }
}
