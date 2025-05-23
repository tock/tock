// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Component for a multi-level feedback queue scheduler.
//!
//! This provides one Component, MLFQComponent.

// Author: Hudson Ayers <hayers@stanford.edu>
// Last modified: 03/31/2020

use core::mem::MaybeUninit;

use capsules_core::virtualizers::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use kernel::component::Component;
use kernel::hil::time;
use kernel::process::ProcessArray;
use kernel::scheduler::mlfq::{MLFQProcessNode, MLFQSched};

#[macro_export]
macro_rules! mlfq_component_static {
    ($A:ty, $N:expr $(,)?) => {{
        let alarm = kernel::static_buf!(
            capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<'static, $A>
        );
        let mlfq_sched = kernel::static_buf!(
            kernel::scheduler::mlfq::MLFQSched<
                'static,
                capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<'static, $A>,
            >
        );
        let mlfq_node = kernel::static_buf!(
            [core::mem::MaybeUninit<kernel::scheduler::mlfq::MLFQProcessNode<'static>>; $N]
        );

        (alarm, mlfq_sched, mlfq_node)
    };};
}

pub struct MLFQComponent<A: 'static + time::Alarm<'static>, const NUM_PROCS: usize> {
    alarm_mux: &'static MuxAlarm<'static, A>,
    processes: &'static ProcessArray<NUM_PROCS>,
}

impl<A: 'static + time::Alarm<'static>, const NUM_PROCS: usize> MLFQComponent<A, NUM_PROCS> {
    pub fn new(
        alarm_mux: &'static MuxAlarm<'static, A>,
        processes: &'static ProcessArray<NUM_PROCS>,
    ) -> MLFQComponent<A, NUM_PROCS> {
        MLFQComponent {
            alarm_mux,
            processes,
        }
    }
}

impl<A: 'static + time::Alarm<'static>, const NUM_PROCS: usize> Component
    for MLFQComponent<A, NUM_PROCS>
{
    type StaticInput = (
        &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
        &'static mut MaybeUninit<MLFQSched<'static, VirtualMuxAlarm<'static, A>>>,
        &'static mut MaybeUninit<[MaybeUninit<MLFQProcessNode<'static>>; NUM_PROCS]>,
    );
    type Output = &'static mut MLFQSched<'static, VirtualMuxAlarm<'static, A>>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let scheduler_alarm = static_buffer.0.write(VirtualMuxAlarm::new(self.alarm_mux));
        scheduler_alarm.setup();

        let scheduler = static_buffer.1.write(MLFQSched::new(scheduler_alarm));

        let nodes = static_buffer
            .2
            .write([const { MaybeUninit::uninit() }; NUM_PROCS]);

        for (i, node) in nodes.iter_mut().enumerate() {
            let init_node = node.write(MLFQProcessNode::new(&self.processes[i]));
            scheduler.processes[0].push_head(init_node);
        }
        scheduler
    }
}
