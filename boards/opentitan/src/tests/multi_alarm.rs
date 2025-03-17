// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Test the behavior of a single alarm.
//! To add this test, include the line
//! ```
//!    multi_alarm_test::run_alarm(alarm_mux);
//! ```
//! to the OpenTitan boot sequence, where `alarm_mux` is a
//! `capsules_core::virtualizers::virtual_alarm::MuxAlarm`. The test sets up 3
//! different virtualized alarms of random durations and prints
//! out when they fire. The durations are uniformly random with
//! one caveat, that 1 in 11 is of duration 0; this is to test
//! that alarms whose expiration was in the past due to the
//! latency of software work correctly.

use crate::tests::run_kernel_op;
use crate::ALARM;
use capsules_core::test::random_alarm::TestRandomAlarm;
use capsules_core::virtualizers::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use earlgrey::timer::RvTimer;
use kernel::hil::time::Alarm;
use kernel::static_init;
use kernel::{debug, non_zero};

static mut TESTS: Option<
    [&'static TestRandomAlarm<
        'static,
        VirtualMuxAlarm<'static, RvTimer<'static, crate::ChipConfig>>,
    >; 3],
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
        assert!(TESTS.unwrap()[2].counter.get() > 70);
    }

    debug!("    [ok]");
    run_kernel_op(100);
}

unsafe fn static_init_multi_alarm_test(
    mux: &'static MuxAlarm<'static, RvTimer<'static, crate::ChipConfig>>,
) -> [&'static TestRandomAlarm<
    'static,
    VirtualMuxAlarm<'static, RvTimer<'static, crate::ChipConfig>>,
>; 3] {
    let virtual_alarm1 = static_init!(
        VirtualMuxAlarm<'static, RvTimer<'static, crate::ChipConfig>>,
        VirtualMuxAlarm::new(mux)
    );
    virtual_alarm1.setup();

    let test1 = static_init!(
        TestRandomAlarm<'static, VirtualMuxAlarm<'static, RvTimer<'static, crate::ChipConfig>>>,
        TestRandomAlarm::new(virtual_alarm1, 19, 'A', false)
    );
    virtual_alarm1.set_alarm_client(test1);

    let virtual_alarm2 = static_init!(
        VirtualMuxAlarm<'static, RvTimer<'static, crate::ChipConfig>>,
        VirtualMuxAlarm::new(mux)
    );
    virtual_alarm2.setup();

    let test2 = static_init!(
        TestRandomAlarm<'static, VirtualMuxAlarm<'static, RvTimer<'static, crate::ChipConfig>>>,
        TestRandomAlarm::new(virtual_alarm2, 37, 'B', false)
    );
    virtual_alarm2.set_alarm_client(test2);

    let virtual_alarm3 = static_init!(
        VirtualMuxAlarm<'static, RvTimer<'static, crate::ChipConfig>>,
        VirtualMuxAlarm::new(mux)
    );
    virtual_alarm3.setup();

    let test3 = static_init!(
        TestRandomAlarm<'static, VirtualMuxAlarm<'static, RvTimer<'static, crate::ChipConfig>>>,
        TestRandomAlarm::new(virtual_alarm3, 89, 'C', false)
    );
    virtual_alarm3.set_alarm_client(test3);
    [&*test1, &*test2, &*test3]
}
