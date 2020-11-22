use core::mem::MaybeUninit;

use capsules::test::random_alarm::TestRandomAlarm;
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use kernel::component::Component;
use kernel::hil::time::{self, Alarm};
use kernel::static_init_half;

#[macro_export]
macro_rules! multi_alarm_test_component_buf {
    ($A:ty $(,)?) => {{
        use capsules::test::random_alarm::TestRandomAlarm;
        use capsules::virtual_alarm::VirtualMuxAlarm;
        use core::mem::MaybeUninit;
        static mut BUF00: MaybeUninit<VirtualMuxAlarm<'static, $A>> = MaybeUninit::uninit();
        static mut BUF01: MaybeUninit<TestRandomAlarm<'static, VirtualMuxAlarm<'static, $A>>> =
            MaybeUninit::uninit();
        static mut BUF10: MaybeUninit<VirtualMuxAlarm<'static, $A>> = MaybeUninit::uninit();
        static mut BUF11: MaybeUninit<TestRandomAlarm<'static, VirtualMuxAlarm<'static, $A>>> =
            MaybeUninit::uninit();
        static mut BUF20: MaybeUninit<VirtualMuxAlarm<'static, $A>> = MaybeUninit::uninit();
        static mut BUF21: MaybeUninit<TestRandomAlarm<'static, VirtualMuxAlarm<'static, $A>>> =
            MaybeUninit::uninit();
        (
            (&mut BUF00, &mut BUF01),
            (&mut BUF10, &mut BUF11),
            (&mut BUF20, &mut BUF21),
        )
    };};
}

pub struct MultiAlarmTestComponent<A: 'static + time::Alarm<'static>> {
    mux: &'static MuxAlarm<'static, A>,
}

impl<A: 'static + time::Alarm<'static>> MultiAlarmTestComponent<A> {
    pub fn new(mux: &'static MuxAlarm<'static, A>) -> Self {
        Self { mux }
    }
}

impl<A: 'static + time::Alarm<'static>> Component for MultiAlarmTestComponent<A> {
    type StaticInput = (
        (
            &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
            &'static mut MaybeUninit<TestRandomAlarm<'static, VirtualMuxAlarm<'static, A>>>,
        ),
        (
            &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
            &'static mut MaybeUninit<TestRandomAlarm<'static, VirtualMuxAlarm<'static, A>>>,
        ),
        (
            &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
            &'static mut MaybeUninit<TestRandomAlarm<'static, VirtualMuxAlarm<'static, A>>>,
        ),
    );
    type Output = MultiAlarmTestRunner<A>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let (buf0, buf1, buf2) = static_buffer;

        let virtual_alarm0 = static_init_half!(
            buf0.0,
            VirtualMuxAlarm<'static, A>,
            VirtualMuxAlarm::new(self.mux)
        );
        let test0 = static_init_half!(
            buf0.1,
            TestRandomAlarm<'static, VirtualMuxAlarm<'static, A>>,
            TestRandomAlarm::new(virtual_alarm0, 19, 'A')
        );
        virtual_alarm0.set_alarm_client(test0);

        let virtual_alarm1 = static_init_half!(
            buf1.0,
            VirtualMuxAlarm<'static, A>,
            VirtualMuxAlarm::new(self.mux)
        );
        let test1 = static_init_half!(
            buf1.1,
            TestRandomAlarm<'static, VirtualMuxAlarm<'static, A>>,
            TestRandomAlarm::new(virtual_alarm1, 37, 'B')
        );
        virtual_alarm1.set_alarm_client(test1);

        let virtual_alarm2 = static_init_half!(
            buf2.0,
            VirtualMuxAlarm<'static, A>,
            VirtualMuxAlarm::new(self.mux)
        );
        let test2 = static_init_half!(
            buf2.1,
            TestRandomAlarm<'static, VirtualMuxAlarm<'static, A>>,
            TestRandomAlarm::new(virtual_alarm2, 89, 'C')
        );
        virtual_alarm2.set_alarm_client(test2);

        MultiAlarmTestRunner {
            tests: [test0, test1, test2],
        }
    }
}

pub struct MultiAlarmTestRunner<A: 'static + time::Alarm<'static>> {
    tests: [&'static TestRandomAlarm<'static, VirtualMuxAlarm<'static, A>>; 3],
}

impl<A: 'static + time::Alarm<'static>> MultiAlarmTestRunner<A> {
    pub fn run(&self) {
        self.tests[0].run();
        self.tests[1].run();
        self.tests[2].run();
    }
}
