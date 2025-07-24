// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Component for a round robin scheduler.
//!
//! This provides one Component, RoundRobinComponent.
//!
//! Usage
//! -----
//! ```rust
//! let scheduler = components::round_robin::RoundRobinComponent::new(&PROCESSES)
//!     .finalize(components::round_robin_component_static!(NUM_PROCS));
//! ```

// Author: Hudson Ayers <hayers@stanford.edu>
// Last modified: 03/31/2020

use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::process::ProcessArray;
use kernel::scheduler::round_robin::{RoundRobinProcessNode, RoundRobinSched};

#[macro_export]
macro_rules! round_robin_component_static {
    ($N:expr $(,)?) => {{
        let rr_sched =
            kernel::static_buf!(kernel::scheduler::round_robin::RoundRobinSched<'static>);
        let rr_nodes = kernel::static_buf!(
            [core::mem::MaybeUninit<kernel::scheduler::round_robin::RoundRobinProcessNode<'static>>;
                $N]
        );

        (rr_sched, rr_nodes)
    };};
}

pub struct RoundRobinComponent<const NUM_PROCS: usize> {
    processes: &'static ProcessArray<NUM_PROCS>,
}

impl<const NUM_PROCS: usize> RoundRobinComponent<NUM_PROCS> {
    pub fn new(processes: &'static ProcessArray<NUM_PROCS>) -> RoundRobinComponent<NUM_PROCS> {
        RoundRobinComponent { processes }
    }
}

impl<const NUM_PROCS: usize> Component for RoundRobinComponent<NUM_PROCS> {
    type StaticInput = (
        &'static mut MaybeUninit<RoundRobinSched<'static>>,
        &'static mut MaybeUninit<[MaybeUninit<RoundRobinProcessNode<'static>>; NUM_PROCS]>,
    );
    type Output = &'static mut RoundRobinSched<'static>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let scheduler = static_buffer.0.write(RoundRobinSched::new());

        let nodes = static_buffer
            .1
            .write([const { MaybeUninit::uninit() }; NUM_PROCS]);

        for (i, node) in nodes.iter_mut().enumerate() {
            let init_node = node.write(RoundRobinProcessNode::new(&self.processes[i]));
            scheduler.processes.push_head(init_node);
        }
        scheduler
    }
}
