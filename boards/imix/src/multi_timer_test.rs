//! Test the behavior of a single alarm.
//! To add this test, include the line
//! ```
//!    multi_timer_test::run_multi_timer(alarm);
//! ```
//! to the imix boot sequence, where `alarm` is a
//! `kernel::hil::Alarm`. The test sets up a series of
//! timer of different durations and prints out when
//! they fire. 

use kernel::debug;
use kernel::hil::time::Timer;
use kernel::static_init;

use capsules::test::random_timer::TestRandomTimer;
use capsules::virtual_timer::{MuxTimer, VirtualTimer};
use sam4l::ast::Ast;

pub unsafe fn run_multi_timer(mux: &'static MuxTimer<'static, Ast<'static>>) {
    debug!("Starting multi timer test.");
    let tests: [&'static TestRandomTimer<'static, VirtualTimer<'static, Ast<'static>>>; 3] =
        static_init_multi_timer_test(mux);
    tests[0].run();
    tests[1].run();
    tests[2].run();
}

unsafe fn static_init_multi_timer_test(
    mux: &'static MuxTimer<'static, Ast<'static>>,
) -> [&'static TestRandomTimer<'static, VirtualTimer<'static, Ast<'static>>>; 3] {
    let virtual_timer1 = static_init!(
        VirtualTimer<'static, Ast<'static>>,
        VirtualTimer::new(mux)
    );
    let test1 = static_init!(
        TestRandomTimer<'static, VirtualTimer<'static, Ast<'static>>>,
        TestRandomTimer::new(virtual_timer1, 19, 'A')
    );
    virtual_timer1.set_timer_client(test1);

    let virtual_timer2 = static_init!(
        VirtualTimer<'static, Ast<'static>>,
        VirtualTimer::new(mux)
    );
    let test2 = static_init!(
        TestRandomTimer<'static, VirtualTimer<'static, Ast<'static>>>,
        TestRandomTimer::new(virtual_timer2, 37, 'B')
    );
    virtual_timer2.set_timer_client(test2);

    let virtual_timer3 = static_init!(
        VirtualTimer<'static, Ast<'static>>,
        VirtualTimer::new(mux)
    );
    let test3 = static_init!(
        TestRandomTimer<'static, VirtualTimer<'static, Ast<'static>>>,
        TestRandomTimer::new(virtual_timer3, 89, 'C')
    );
    virtual_timer3.set_timer_client(test3);
    [&*test1, &*test2, &*test3]
}
