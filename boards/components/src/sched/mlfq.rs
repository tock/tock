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
    ($A:ty, $N:expr) => {{
        use core::mem::MaybeUninit;
        use kernel::static_init;
        use kernel::{MLFQProcessNode, MLFQSched};
        static mut BUF1: MaybeUninit<VirtualMuxAlarm<'static, $A>> = MaybeUninit::uninit();
        static mut BUF2: MaybeUninit<MLFQSched<'static, VirtualMuxAlarm<'static, $A>>> =
            MaybeUninit::uninit();
        (
            &mut BUF1,
            &mut BUF2,
            static_init!([Option<MLFQProcessNode<'static>>; $N], [None; $N]),
        )
    };};
}

pub struct MLFQComponent<A: 'static + time::Alarm<'static>> {
    board_kernel: &'static kernel::Kernel,
    alarm_mux: &'static MuxAlarm<'static, A>,
    processes: &'static [Option<&'static dyn ProcessType>],
}

impl<A: 'static + time::Alarm<'static>> MLFQComponent<A> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        alarm_mux: &'static MuxAlarm<'static, A>,
        processes: &'static [Option<&'static dyn ProcessType>],
    ) -> MLFQComponent<A> {
        MLFQComponent {
            board_kernel: board_kernel,
            alarm_mux: alarm_mux,
            processes: processes,
        }
    }
}

impl<A: 'static + time::Alarm<'static>> Component for MLFQComponent<A> {
    type StaticInput = (
        &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
        &'static mut MaybeUninit<MLFQSched<'static, VirtualMuxAlarm<'static, A>>>,
        &'static mut [Option<MLFQProcessNode<'static>>],
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
            MLFQSched::new(self.board_kernel, scheduler_alarm)
        );
        let num_procs = proc_nodes.len();

        for i in 0..num_procs {
            if self.processes[i].is_some() {
                proc_nodes[i] = Some(MLFQProcessNode::new(self.processes[i].unwrap().appid()));
            }
        }
        for i in 0..num_procs {
            if self.processes[i].is_some() {
                scheduler.processes[0].push_head(proc_nodes[i].as_ref().unwrap());
            }
        }
        scheduler
    }
}
