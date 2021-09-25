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
use kernel::collections::list::simple_linked_list::{SimpleLinkedList, SimpleLinkedListNode};
use kernel::collections::list::SinglyLinkedList;
use kernel::component::Component;
use kernel::process::Process;
use kernel::scheduler::round_robin::RoundRobinSched;
use kernel::{static_init, static_init_half};

#[macro_export]
macro_rules! rr_component_helper {
    ($N:expr $(,)?) => {{
        use core::mem::MaybeUninit;
        use kernel::collections::list::simple_linked_list::SimpleLinkedListNode;
        use kernel::process::Process;
        use kernel::static_buf;
        const UNINIT: MaybeUninit<SimpleLinkedListNode<'static, Option<&'static dyn Process>>> =
            MaybeUninit::uninit();
        static mut BUF: [MaybeUninit<SimpleLinkedListNode<'static, Option<&'static dyn Process>>>;
            $N] = [UNINIT; $N];
        &mut BUF
    };};
}

pub type SchedulerType = RoundRobinSched<
                'static,
                SimpleLinkedListNode<'static, Option<&'static dyn Process>>,
                SimpleLinkedList<'static, Option<&'static dyn Process>>,
            >;

pub struct RoundRobinComponent {
    processes: &'static [Option<&'static dyn Process>],
}

impl RoundRobinComponent {
    pub fn new(processes: &'static [Option<&'static dyn Process>]) -> RoundRobinComponent {
        RoundRobinComponent { processes }
    }
}

impl Component for RoundRobinComponent {
    type StaticInput =
        &'static mut [MaybeUninit<SimpleLinkedListNode<'static, Option<&'static dyn Process>>>];
    type Output = &'static mut SchedulerType;

    unsafe fn finalize(self, buf: Self::StaticInput) -> Self::Output {
        let scheduler = static_init!(
            SchedulerType,
            RoundRobinSched::new(SimpleLinkedList::new())
        );

        for (i, node) in buf.iter_mut().enumerate() {
            let init_node = static_init_half!(
                node,
                SimpleLinkedListNode<'static, Option<&'static dyn Process>>,
                SimpleLinkedListNode::new(self.processes[i])
            );
            scheduler.processes.push_head(init_node);
        }
        scheduler
    }
}
