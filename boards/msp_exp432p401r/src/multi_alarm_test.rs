//! Test the behavior of a single alarm.
//! To add this test, include the line
//! ```
//!    multi_alarm_test::run_alarm(alarm_mux);
//! ```
//! to the boot sequence, where `alarm_mux` is a
//! `capsules::virtual_alarm::MuxAlarm`. The test sets up 3
//! different virtualized alarms of random durations and prints
//! out when they fire. The durations are uniformly random with
//! one caveat, that 1 in 11 is of duration 0; this is to test
//! that alarms whose expiration was in the past due to the
//! latency of software work correctly.

use capsules::test::random_alarm::TestRandomAlarm;
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use kernel::debug;
use kernel::hil::time::Alarm;
use kernel::static_init;
use msp432::timer::TimerA;

pub unsafe fn run_multi_alarm(mux: &'static MuxAlarm<'static, TimerA<'static>>) {
    debug!("Starting multi alarm test.");
    let tests: [&'static TestRandomAlarm<'static, VirtualMuxAlarm<'static, TimerA<'static>>>; 3] =
        static_init_multi_alarm_test(mux);
    tests[0].run();
    tests[1].run();
    tests[2].run();
}

unsafe fn static_init_multi_alarm_test(
    mux: &'static MuxAlarm<'static, TimerA<'static>>,
) -> [&'static TestRandomAlarm<'static, VirtualMuxAlarm<'static, TimerA<'static>>>; 3] {
    let virtual_alarm1 = static_init!(
        VirtualMuxAlarm<'static, TimerA<'static>>,
        VirtualMuxAlarm::new(mux)
    );
    let test1 = static_init!(
        TestRandomAlarm<'static, VirtualMuxAlarm<'static, TimerA<'static>>>,
        TestRandomAlarm::new(virtual_alarm1, 19, 'A')
    );
    virtual_alarm1.set_alarm_client(test1);

    let virtual_alarm2 = static_init!(
        VirtualMuxAlarm<'static, TimerA<'static>>,
        VirtualMuxAlarm::new(mux)
    );
    let test2 = static_init!(
        TestRandomAlarm<'static, VirtualMuxAlarm<'static, TimerA<'static>>>,
        TestRandomAlarm::new(virtual_alarm2, 37, 'B')
    );
    virtual_alarm2.set_alarm_client(test2);

    let virtual_alarm3 = static_init!(
        VirtualMuxAlarm<'static, TimerA<'static>>,
        VirtualMuxAlarm::new(mux)
    );
    let test3 = static_init!(
        TestRandomAlarm<'static, VirtualMuxAlarm<'static, TimerA<'static>>>,
        TestRandomAlarm::new(virtual_alarm3, 89, 'C')
    );
    virtual_alarm3.set_alarm_client(test3);
    [&*test1, &*test2, &*test3]
}
