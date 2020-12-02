//! Component for a multi-level feedback queue scheduler.
//!
//! This provides one Component, MLFQComponent.

// Author: Hudson Ayers <hayers@stanford.edu>
// Last modified: 03/31/2020

use core::mem::MaybeUninit;

use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use kernel::component::Component;
use kernel::hil::time;
use kernel::procs::ProcessType;
use kernel::static_init_half;
use kernel::{MLFQProcessNode, MLFQSched};

#[macro_export]
macro_rules! mlfq_component_helper {
    ($A:ty, $N:expr $(,)?) => {{
        use core::mem::MaybeUninit;
        use kernel::static_init;
        use kernel::{MLFQProcessNode, MLFQSched};
        static mut BUF1: MaybeUninit<VirtualMuxAlarm<'static, $A>> = MaybeUninit::uninit();
        static mut BUF2: MaybeUninit<MLFQSched<'static, VirtualMuxAlarm<'static, $A>>> =
            MaybeUninit::uninit();
        static mut BUF3: [MaybeUninit<MLFQProcessNode<'static>>; $N] = [MaybeUninit::uninit(); $N];
        (&mut BUF1, &mut BUF2, &mut BUF3)
    };};
}

pub struct MLFQComponent<A: 'static + time::Alarm<'static>> {
    alarm_mux: &'static MuxAlarm<'static, A>,
    processes: &'static [Option<&'static dyn ProcessType>],
}

impl<A: 'static + time::Alarm<'static>> MLFQComponent<A> {
    pub fn new(
        alarm_mux: &'static MuxAlarm<'static, A>,
        processes: &'static [Option<&'static dyn ProcessType>],
    ) -> MLFQComponent<A> {
        MLFQComponent {
            alarm_mux,
            processes,
        }
    }
}

impl<A: 'static + time::Alarm<'static>> Component for MLFQComponent<A> {
    type StaticInput = (
        &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
        &'static mut MaybeUninit<MLFQSched<'static, VirtualMuxAlarm<'static, A>>>,
        &'static mut [MaybeUninit<MLFQProcessNode<'static>>],
    );
    type Output = &'static mut MLFQSched<'static, VirtualMuxAlarm<'static, A>>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let (alarm_buf, sched_buf, proc_nodes) = static_buffer;
        let scheduler_alarm = static_init_half!(
            alarm_buf,
            VirtualMuxAlarm<'static, A>,
            VirtualMuxAlarm::new(self.alarm_mux)
        );
        let scheduler = static_init_half!(
            sched_buf,
            MLFQSched<'static, VirtualMuxAlarm<'static, A>>,
            MLFQSched::new(scheduler_alarm)
        );
        for (i, node) in proc_nodes.iter_mut().enumerate() {
            let init_node = static_init_half!(
                node,
                MLFQProcessNode<'static>,
                MLFQProcessNode::new(&self.processes[i])
            );
            scheduler.processes[0].push_head(init_node);
        }
        scheduler
    }
}
