//! Test the behavior of a single alarm.
//! To add this test, include the line
//! ```
//!    alarm_test::run_alarm(alarm);
//! ```
//! to the imix boot sequence, where `alarm` is a
//! `kernel::hil::Alarm`. The test sets up a series of
//! alarms of different durations and prints out when
//! they fire. They are large enough (and spaced out
//! enough that you should be able to tell if things
//! are working reasonably well. The module also uses
//! debug_gpio on pin XX so you can more precisely check
//! the timings with a logic analyzer.

use kernel::debug;
use kernel::hil::time::Alarm;
use kernel::static_init;

use capsules::test::random_alarm::TestRandomAlarm;
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use sam4l::ast::Ast;

pub unsafe fn run_multi_alarm(mux: &'static MuxAlarm<'static, Ast<'static>>) {
    debug!("Starting multi alarm test.");
    let tests: [&'static TestRandomAlarm<'static, VirtualMuxAlarm<'static, Ast<'static>>>; 3] =
        static_init_multi_alarm_test(mux);
    tests[0].run();
    tests[1].run();
    tests[2].run();
}

unsafe fn static_init_multi_alarm_test(
    mux: &'static MuxAlarm<'static, Ast<'static>>,
) -> [&'static TestRandomAlarm<'static, VirtualMuxAlarm<'static, Ast<'static>>>; 3] {
    let virtual_alarm1 = static_init!(
        VirtualMuxAlarm<'static, Ast<'static>>,
        VirtualMuxAlarm::new(mux)
    );
    let test1 = static_init!(
        TestRandomAlarm<'static, VirtualMuxAlarm<'static, Ast<'static>>>,
        TestRandomAlarm::new(virtual_alarm1, 19, 'A')
    );
    virtual_alarm1.set_alarm_client(test1);

    let virtual_alarm2 = static_init!(
        VirtualMuxAlarm<'static, Ast<'static>>,
        VirtualMuxAlarm::new(mux)
    );
    let test2 = static_init!(
        TestRandomAlarm<'static, VirtualMuxAlarm<'static, Ast<'static>>>,
        TestRandomAlarm::new(virtual_alarm2, 37, 'B')
    );
    virtual_alarm2.set_alarm_client(test2);

    let virtual_alarm3 = static_init!(
        VirtualMuxAlarm<'static, Ast<'static>>,
        VirtualMuxAlarm::new(mux)
    );
    let test3 = static_init!(
        TestRandomAlarm<'static, VirtualMuxAlarm<'static, Ast<'static>>>,
        TestRandomAlarm::new(virtual_alarm3, 89, 'C')
    );
    virtual_alarm3.set_alarm_client(test3);
    [&*test1, &*test2, &*test3]
}
