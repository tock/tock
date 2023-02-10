use core::mem::MaybeUninit;

use core_capsules::test::random_alarm::TestRandomAlarm;
use core_capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use kernel::component::Component;
use kernel::hil::time::{self, Alarm};

#[macro_export]
macro_rules! multi_alarm_test_component_buf {
    ($A:ty $(,)?) => {{
        use core_capsules::test::random_alarm::TestRandomAlarm;
        use core_capsules::virtual_alarm::VirtualMuxAlarm;

        let buf00 = kernel::static_buf!(VirtualMuxAlarm<'static, $A>);
        let buf01 = kernel::static_buf!(TestRandomAlarm<'static, VirtualMuxAlarm<'static, $A>>);
        let buf10 = kernel::static_buf!(VirtualMuxAlarm<'static, $A>);
        let buf11 = kernel::static_buf!(TestRandomAlarm<'static, VirtualMuxAlarm<'static, $A>>);
        let buf20 = kernel::static_buf!(VirtualMuxAlarm<'static, $A>);
        let buf21 = kernel::static_buf!(TestRandomAlarm<'static, VirtualMuxAlarm<'static, $A>>);

        ((buf00, buf01)(buf10, buf11)(buf20, buf21))
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

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let (buf0, buf1, buf2) = static_buffer;

        let virtual_alarm0 = buf0.0.write(VirtualMuxAlarm::new(self.mux));
        virtual_alarm0.setup();

        let test0 = buf0
            .1
            .write(TestRandomAlarm::new(virtual_alarm0, 19, 'A', true));
        virtual_alarm0.set_alarm_client(test0);

        let virtual_alarm1 = buf1.0.write(VirtualMuxAlarm::new(self.mux));
        virtual_alarm1.setup();

        let test1 = buf1
            .1
            .write(TestRandomAlarm::new(virtual_alarm1, 37, 'B', true));
        virtual_alarm1.set_alarm_client(test1);

        let virtual_alarm2 = buf2.0.write(VirtualMuxAlarm::new(self.mux));
        virtual_alarm2.setup();

        let test2 = buf2
            .1
            .write(TestRandomAlarm::new(virtual_alarm2, 89, 'C', true));
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
