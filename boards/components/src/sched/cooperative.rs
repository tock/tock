//! Component for a cooperative scheduler.
//!
//! This provides one Component, CooperativeComponent.
//!
//! Usage
//! -----
//! ```rust
//! let scheduler = components::cooperative::CooperativeComponent::new(&PROCESSES)
//!     .finalize(components::coop_component_helper!(NUM_PROCS));
//! ```

// Author: Hudson Ayers <hayers@stanford.edu>

use kernel::component::Component;
use kernel::procs::ProcessType;
use kernel::static_init;
use kernel::{CoopProcessNode, CooperativeSched};

#[macro_export]
macro_rules! coop_component_helper {
    ($N:expr) => {{
        use kernel::static_init;
        use kernel::CoopProcessNode;
        static_init!([Option<CoopProcessNode<'static>>; $N], [None; $N])
    };};
}

pub struct CooperativeComponent {
    processes: &'static [Option<&'static dyn ProcessType>],
}

impl CooperativeComponent {
    pub fn new(processes: &'static [Option<&'static dyn ProcessType>]) -> CooperativeComponent {
        CooperativeComponent { processes }
    }
}

impl Component for CooperativeComponent {
    type StaticInput = &'static mut [Option<CoopProcessNode<'static>>];
    type Output = &'static mut CooperativeSched<'static>;

    unsafe fn finalize(self, proc_nodes: Self::StaticInput) -> Self::Output {
        let scheduler = static_init!(CooperativeSched<'static>, CooperativeSched::new());
        let num_procs = proc_nodes.len();

        for i in 0..num_procs {
            if self.processes[i].is_some() {
                proc_nodes[i] = Some(CoopProcessNode::new(self.processes[i].unwrap().appid()));
            }
        }
        for i in 0..num_procs {
            if self.processes[i].is_some() {
                scheduler
                    .processes
                    .push_head(proc_nodes[i].as_ref().unwrap());
            }
        }
        scheduler
    }
}
