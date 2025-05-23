// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Component for a cooperative scheduler.
//!
//! This provides one Component, CooperativeComponent.
//!
//! Usage
//! -----
//! ```rust
//! let scheduler = components::cooperative::CooperativeComponent::new(&PROCESSES)
//!     .finalize(components::cooperative_component_static!(NUM_PROCS));
//! ```

// Author: Hudson Ayers <hayers@stanford.edu>

use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::process::ProcessArray;
use kernel::scheduler::cooperative::{CoopProcessNode, CooperativeSched};

#[macro_export]
macro_rules! cooperative_component_static {
    ($N:expr $(,)?) => {{
        let coop_sched =
            kernel::static_buf!(kernel::scheduler::cooperative::CooperativeSched<'static>);
        let coop_nodes = kernel::static_buf!(
            [core::mem::MaybeUninit<kernel::scheduler::cooperative::CoopProcessNode<'static>>; $N]
        );

        (coop_sched, coop_nodes)
    };};
}

pub struct CooperativeComponent<const NUM_PROCS: usize> {
    processes: &'static ProcessArray<NUM_PROCS>,
}

impl<const NUM_PROCS: usize> CooperativeComponent<NUM_PROCS> {
    pub fn new(processes: &'static ProcessArray<NUM_PROCS>) -> CooperativeComponent<NUM_PROCS> {
        CooperativeComponent { processes }
    }
}

impl<const NUM_PROCS: usize> Component for CooperativeComponent<NUM_PROCS> {
    type StaticInput = (
        &'static mut MaybeUninit<CooperativeSched<'static>>,
        &'static mut MaybeUninit<[MaybeUninit<CoopProcessNode<'static>>; NUM_PROCS]>,
    );
    type Output = &'static mut CooperativeSched<'static>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let scheduler = static_buffer.0.write(CooperativeSched::new());

        let nodes = static_buffer
            .1
            .write([const { MaybeUninit::uninit() }; NUM_PROCS]);

        for (i, node) in nodes.iter_mut().enumerate() {
            let init_node = node.write(CoopProcessNode::new(&self.processes[i]));
            scheduler.processes.push_head(init_node);
        }
        scheduler
    }
}
