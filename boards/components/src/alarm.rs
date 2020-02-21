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

// Setup static space for the objects.
#[macro_export]
macro_rules! alarm_mux_component_buf {
    ($A:ty) => {{
        use capsules::virtual_alarm::MuxAlarm;
        $crate::uninit_static_buf!(MuxAlarm<'static, $A>)
    };};
}

// Setup static space for the objects.
#[macro_export]
macro_rules! alarm_component_buf {
    ($A:ty) => {{
        use capsules::alarm::AlarmDriver;
        use capsules::virtual_alarm::VirtualMuxAlarm;
        (
            $crate::uninit_static_buf!(VirtualMuxAlarm<'static, $A>),
            $crate::uninit_static_buf!(AlarmDriver<'static, VirtualMuxAlarm<'static, $A>>),
        )
    };};
}

pub mod alarm_mux_component {
    use capsules::virtual_alarm::MuxAlarm;
    use kernel::hil::time;
    use kernel::UninitStaticBuf;

    pub unsafe fn create<A: 'static + time::Alarm<'static>>(
        alarm: &'static A,
        static_buffer: UninitStaticBuf<MuxAlarm<'static, A>>,
    ) -> &'static MuxAlarm<'static, A> {
        let mux_alarm = static_buffer.static_init(MuxAlarm::new(alarm));

        time::Alarm::set_client(alarm, mux_alarm);
        mux_alarm
    }
}

pub mod alarm_driver_component {
    use capsules::alarm::AlarmDriver;
    use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
    use kernel::hil::time;
    use kernel::UninitStaticBuf;
    use kernel::{capabilities, create_capability};

    pub unsafe fn create<A: 'static + time::Alarm<'static>>(
        board_kernel: &'static kernel::Kernel,
        alarm_mux: &'static MuxAlarm<'static, A>,
        static_buffer: (
            UninitStaticBuf<VirtualMuxAlarm<'static, A>>,
            UninitStaticBuf<AlarmDriver<'static, VirtualMuxAlarm<'static, A>>>,
        ),
    ) -> &'static AlarmDriver<'static, VirtualMuxAlarm<'static, A>> {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let virtual_alarm1 = static_buffer.0.static_init(VirtualMuxAlarm::new(alarm_mux));
        let alarm = static_buffer.1.static_init(AlarmDriver::new(
            virtual_alarm1,
            board_kernel.create_grant(&grant_cap),
        ));

        time::Alarm::set_client(virtual_alarm1, alarm);
        alarm
    }
}
