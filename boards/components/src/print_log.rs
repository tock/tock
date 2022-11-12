//! Components for PrintLog, a system call driver that outputs
//! userspace printf statements into the debug output buffer of the
//! kernel; this has the property that userspace and kernel print
//! statements are temporally ordered (unlike in Console, where they
//! can be reordered).
//!
//! This provides one Component, `PrintComponent`, which which
//! implements the same printing system call API as Console but puts
//! strings into the debug queue.  Prints from userspace that are too
//! long for the queue are truncated.
//!
//! Usage
//! -----
//! ```rust
//! let ast = &sam4l::ast::AST;
//! let mux_alarm = components::alarm::AlarmMuxComponent::new(ast)
//!     .finalize(components::alarm_mux_component_static!(sam4l::ast::Ast));
//! ast.configure(mux_alarm);
//! let print_log = components::print_log::PrintLogComponent::new(board_kernel, driver_num, mux_alarm)
//!     .finalize(components::printlog_component_static!(sam4l::ast::Ast));


//! ```
// Author: Philip Levis <pal@cs.stanford.edu>
// Last modified: 1/08/2020

use capsules::print_log::PrintLog;
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::time::{self, Alarm};

#[macro_export]
macro_rules! printlog_component_static {
    ($A:ty $(,)?) => {{
        let mux_alarm = kernel::static_buf!(VirtualMuxAlarm<'static, $A>);
        let print_log = kernel::static_buf!(
            PrintLog<'static, VirtualMuxAlarm<'static, $A>,
            >
        );
        (mux_alarm, print_log)
    };};
}

pub struct PrintLogComponent<A: 'static + time::Alarm<'static>> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    alarm_mux: &'static MuxAlarm<'static, A>,
}

impl<A: 'static + time::Alarm<'static>> PrintLogComponent<A> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        alarm_mux: &'static MuxAlarm<'static, A>,
    ) -> PrintLogComponent<A>{
        PrintLogComponent {
            board_kernel: board_kernel,
            driver_num: driver_num,
            alarm_mux: alarm_mux
        }
    }
}

impl<A: 'static + time::Alarm<'static>> Component for PrintLogComponent<A> {
    type StaticInput = (
        &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
        &'static mut MaybeUninit<PrintLog<'static, VirtualMuxAlarm<'static, A>>>,
    );
    type Output = &'static PrintLog<'static, VirtualMuxAlarm<'static, A>>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let virtual_alarm1 = static_buffer.0.write(VirtualMuxAlarm::new(self.alarm_mux));
        virtual_alarm1.setup();

        let print_log = static_buffer.1.write(PrintLog::new(
            virtual_alarm1,
            self.board_kernel.create_grant(self.driver_num, &grant_cap),
        ));
        
        virtual_alarm1.set_alarm_client(print_log);
        print_log
    }
}
