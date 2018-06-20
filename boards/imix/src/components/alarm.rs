//! Component for harware timer Alarms on the imix board.
//!
//! This provides one component, AlarmDriverComponent, which provides
//! an alarm system call interface.
//!
//! Usage
//! -----
//! ```rust
//! let alarm = AlarmDriverComponent::new(mux_alarm).finalize();
//! ```

// Author: Philip Levis <pal@cs.stanford.edu>
// Last modified: 6/20/2018

#![allow(dead_code)] // Components are intended to be conditionally included

use sam4l;
use capsules::alarm::AlarmDriver;
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use kernel;
use kernel::component::Component;

pub struct AlarmDriverComponent {
    alarm_mux: &'static MuxAlarm<'static, sam4l::ast::Ast<'static>>,
}

impl AlarmDriverComponent {
    pub fn new(mux: &'static MuxAlarm<'static, sam4l::ast::Ast>) -> AlarmDriverComponent {
        AlarmDriverComponent {
            alarm_mux: mux
        }
    }
}

impl Component for AlarmDriverComponent {
    type Output = &'static AlarmDriver<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast<'static>>>;

    unsafe fn finalize(&mut self) -> Self::Output {
        let virtual_alarm1 = static_init!(
            VirtualMuxAlarm<'static, sam4l::ast::Ast>,
            VirtualMuxAlarm::new(self.alarm_mux)
        );
        let alarm = static_init!(
            AlarmDriver<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast>>,
            AlarmDriver::new(virtual_alarm1, kernel::Grant::create())
        );

        virtual_alarm1.set_client(alarm);
        alarm
    }
}
