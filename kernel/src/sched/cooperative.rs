//! Cooperative Scheduler for Tock
//!
//! When hardware interrupts occur while a userspace process is executing,
//! this scheduler executes the top half of the interrupt,
//! and then stops executing the userspace process immediately and handles the bottom
//! half of the interrupt. However it then continues executing the same userspace process
//! that was executing. This scheduler overwrites the systick

use crate::callback::AppId;
use crate::common::list::{List, ListLink, ListNode};
use crate::platform::Chip;
use crate::sched::{Kernel, Scheduler, StoppedExecutingReason};

/// A node in the linked list the scheduler uses to track processes
pub struct CoopProcessNode<'a> {
    appid: AppId,
    next: ListLink<'a, CoopProcessNode<'a>>,
}

impl<'a> CoopProcessNode<'a> {
    pub fn new(appid: AppId) -> CoopProcessNode<'a> {
        CoopProcessNode {
            appid,
            next: ListLink::empty(),
        }
    }
}

impl<'a> ListNode<'a, CoopProcessNode<'a>> for CoopProcessNode<'a> {
    fn next(&'a self) -> &'a ListLink<'a, CoopProcessNode> {
        &self.next
    }
}

/// Cooperative Scheduler
pub struct CooperativeSched<'a> {
    pub processes: List<'a, CoopProcessNode<'a>>,
}

impl<'a> CooperativeSched<'a> {
    pub const fn new() -> CooperativeSched<'a> {
        CooperativeSched {
            processes: List::new(),
        }
    }
}

impl<'a, C: Chip> Scheduler<C> for CooperativeSched<'a> {
    fn next(&self, kernel: &Kernel) -> (Option<AppId>, Option<u32>) {
        if kernel.processes_blocked() {
            // No processes ready
            (None, None)
        } else {
            let next = self.processes.head().unwrap().appid;

            (Some(next), None)
        }
    }

    fn result(&self, result: StoppedExecutingReason, _: Option<u32>) {
        let reschedule = match result {
            StoppedExecutingReason::KernelPreemption => true,
            _ => false,
        };
        if !reschedule {
            self.processes.push_tail(self.processes.pop_head().unwrap());
        }
    }
}
