//! Component for a multi-level feedback queue scheduler.
//!
//! This provides one Component, MLFQComponent.

// Author: Hudson Ayers <hayers@stanford.edu>
// Last modified: 03/31/2020

use core::mem::MaybeUninit;

use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use kernel::collections::list::simple_linked_list::{SimpleLinkedList, SimpleLinkedListNode};
use kernel::collections::list::SinglyLinkedList;
use kernel::component::Component;
use kernel::hil::time;
use kernel::process::Process;
use kernel::scheduler::mlfq::{MLFQProcessState, MLFQSched};
use kernel::static_init_half;

#[macro_export]
macro_rules! mlfq_component_helper {
    ($A:ty, $N:expr $(,)?) => {{
        use core::mem::MaybeUninit;
        use kernel::collections::list::simple_linked_list::SimpleLinkedListNode;
        use kernel::scheduler::mlfq::{MLFQProcessState, MLFQSched};
        use kernel::static_init;
        static mut BUF1: MaybeUninit<VirtualMuxAlarm<'static, $A>> = MaybeUninit::uninit();
        static mut BUF2: MaybeUninit<MLFQSched<'static, VirtualMuxAlarm<'static, $A>>> =
            MaybeUninit::uninit();
        const UNINIT: MaybeUninit<SimpleLinkedListNode<'static, MLFQProcessState>> =
            MaybeUninit::uninit();
        static mut BUF3: [MaybeUninit<SimpleLinkedListNode<'static, MLFQProcessState>>; $N] =
            [UNINIT; $N];
        (&mut BUF1, &mut BUF2, &mut BUF3)
    };};
}

pub type SchedulerType<A> = MLFQSched<
                'static,
                VirtualMuxAlarm<'static, A>,
                SimpleLinkedListNode<'static, MLFQProcessState>,
                SimpleLinkedList<'static, MLFQProcessState>,
            >;

pub struct MLFQComponent<A: 'static + time::Alarm<'static>> {
    alarm_mux: &'static MuxAlarm<'static, A>,
    processes: &'static [Option<&'static dyn Process>],
}

impl<A: 'static + time::Alarm<'static>> MLFQComponent<A> {
    pub fn new(
        alarm_mux: &'static MuxAlarm<'static, A>,
        processes: &'static [Option<&'static dyn Process>],
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
        &'static mut MaybeUninit<
            SchedulerType<A>
        >,
        &'static mut [MaybeUninit<SimpleLinkedListNode<'static, MLFQProcessState>>],
    );
    type Output = &'static mut SchedulerType<A>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let (alarm_buf, sched_buf, proc_nodes) = static_buffer;
        let scheduler_alarm = static_init_half!(
            alarm_buf,
            VirtualMuxAlarm<'static, A>,
            VirtualMuxAlarm::new(self.alarm_mux)
        );
        scheduler_alarm.setup();

        let scheduler = static_init_half!(
            sched_buf,
            SchedulerType<A>,
            MLFQSched::new(
                scheduler_alarm,
                [
                    SimpleLinkedList::new(),
                    SimpleLinkedList::new(),
                    SimpleLinkedList::new()
                ]
            )
        );
        for (i, node) in proc_nodes.iter_mut().enumerate() {
            let init_node = static_init_half!(
                node,
                SimpleLinkedListNode<'static, MLFQProcessState>,
                SimpleLinkedListNode::new(MLFQProcessState::new(self.processes[i]))
            );
            scheduler.processes[0].push_head(init_node);
        }
        scheduler
    }
}
