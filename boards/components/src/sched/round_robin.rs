//! Component for a round robin scheduler.
//!
//! This provides one Component, RoundRobinComponent.
//!
//! Usage
//! -----
//! ```rust
//! let scheduler = components::round_robin::RoundRobinComponent::new(&PROCESSES)
//!     .finalize(components::rr_component_helper!(NUM_PROCS));
//! ```

// Author: Hudson Ayers <hayers@stanford.edu>
// Last modified: 03/31/2020

use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::procs::ProcessType;
use kernel::{static_init, static_init_half};
use kernel::{RoundRobinProcessNode, RoundRobinSched};

#[macro_export]
macro_rules! rr_component_helper {
    ($N:expr) => {{
        use core::mem::MaybeUninit;
        use kernel::static_buf;
        use kernel::RoundRobinProcessNode;
        static mut BUF: [MaybeUninit<RoundRobinProcessNode<'static>>; $N] =
            [MaybeUninit::uninit(); $N];
        &mut BUF
    };};
}

pub struct RoundRobinComponent {
    processes: &'static [Option<&'static dyn ProcessType>],
}

impl RoundRobinComponent {
    pub fn new(processes: &'static [Option<&'static dyn ProcessType>]) -> RoundRobinComponent {
        RoundRobinComponent { processes }
    }
}

impl Component for RoundRobinComponent {
    type StaticInput = &'static mut [MaybeUninit<RoundRobinProcessNode<'static>>];
    type Output = &'static mut RoundRobinSched<'static>;

    unsafe fn finalize(self, buf: Self::StaticInput) -> Self::Output {
        let scheduler = static_init!(RoundRobinSched<'static>, RoundRobinSched::new());

        for (i, node) in buf.iter_mut().enumerate() {
            let init_node = static_init_half!(
                node,
                RoundRobinProcessNode<'static>,
                RoundRobinProcessNode::new(&self.processes[i])
            );
            scheduler.processes.push_head(init_node);
        }
        scheduler
    }
}
