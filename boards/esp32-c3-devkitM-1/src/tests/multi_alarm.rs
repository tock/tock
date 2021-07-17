//! Test the behavior of a single alarm.
//! To add this test, include the line
//! ```
//!    multi_alarm_test::run_alarm(alarm_mux);
//! ```
//! to the OpenTitan boot sequence, where `alarm_mux` is a
//! `capsules::virtual_alarm::MuxAlarm`. The test sets up 3
//! different virtualized alarms of random durations and prints
//! out when they fire. The durations are uniformly random with
//! one caveat, that 1 in 11 is of duration 0; this is to test
//! that alarms whose expiration was in the past due to the
//! latency of software work correctly.

use crate::tests::run_kernel_op;
use crate::ALARM;
use capsules::test::random_alarm::TestRandomAlarm;
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use esp32::timg::TimG;
use kernel::debug;
use kernel::hil::time::Alarm;
use kernel::static_init;

static mut TESTS: Option<
    [&'static TestRandomAlarm<'static, VirtualMuxAlarm<'static, TimG<'static>>>; 3],
> = None;

#[test_case]
pub fn run_multi_alarm() {
    debug!("start multi alarm test...");
    unsafe {
        TESTS = Some(static_init_multi_alarm_test(ALARM.unwrap()));
        TESTS.unwrap()[0].run();
        TESTS.unwrap()[1].run();
        TESTS.unwrap()[2].run();
    }

    run_kernel_op(10000);

    unsafe {
        assert!(TESTS.unwrap()[0].counter.get() > 15);
        assert!(TESTS.unwrap()[1].counter.get() > 30);
        assert!(TESTS.unwrap()[2].counter.get() > 80);
    }

    debug!("    [ok]");
    run_kernel_op(100);
}

unsafe fn static_init_multi_alarm_test(
    mux: &'static MuxAlarm<'static, TimG<'static>>,
) -> [&'static TestRandomAlarm<'static, VirtualMuxAlarm<'static, TimG<'static>>>; 3] {
    let virtual_alarm1 = static_init!(
        VirtualMuxAlarm<'static, TimG<'static>>,
        VirtualMuxAlarm::new(mux)
    );
    let test1 = static_init!(
        TestRandomAlarm<'static, VirtualMuxAlarm<'static, TimG<'static>>>,
        TestRandomAlarm::new(virtual_alarm1, 19, 'A')
    );
    virtual_alarm1.set_alarm_client(test1);

    let virtual_alarm2 = static_init!(
        VirtualMuxAlarm<'static, TimG<'static>>,
        VirtualMuxAlarm::new(mux)
    );
    let test2 = static_init!(
        TestRandomAlarm<'static, VirtualMuxAlarm<'static, TimG<'static>>>,
        TestRandomAlarm::new(virtual_alarm2, 37, 'B')
    );
    virtual_alarm2.set_alarm_client(test2);

    let virtual_alarm3 = static_init!(
        VirtualMuxAlarm<'static, TimG<'static>>,
        VirtualMuxAlarm::new(mux)
    );
    let test3 = static_init!(
        TestRandomAlarm<'static, VirtualMuxAlarm<'static, TimG<'static>>>,
        TestRandomAlarm::new(virtual_alarm3, 89, 'C')
    );
    virtual_alarm3.set_alarm_client(test3);
    [&*test1, &*test2, &*test3]
}
