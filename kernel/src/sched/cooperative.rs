//! Cooperative Scheduler for Tock
//!
//! This scheduler runs all processes in a round-robin fashion, but does not use
//! a scheduler timer to enforce process timeslices. That is, all processes are
//! run cooperatively. Processes are run until they yield or stop executing
//! (i.e. they crash or exit).
//!
//! When hardware interrupts occur while a userspace process is executing, this
//! scheduler executes the top half of the interrupt, and then stops executing
//! the userspace process immediately and handles the bottom half of the
//! interrupt. However it then continues executing the same userspace process
//! that was executing.

use crate::common::list::{List, ListLink, ListNode};
use crate::platform::Chip;
use crate::process::ProcessType;
use crate::sched::{Kernel, Scheduler, SchedulingDecision, StoppedExecutingReason};

/// A node in the linked list the scheduler uses to track processes
pub struct CoopProcessNode<'a> {
    proc: &'static Option<&'static dyn ProcessType>,
    next: ListLink<'a, CoopProcessNode<'a>>,
}

impl<'a> CoopProcessNode<'a> {
    pub fn new(proc: &'static Option<&'static dyn ProcessType>) -> CoopProcessNode<'a> {
        CoopProcessNode {
            proc,
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
    fn next(&self, kernel: &Kernel) -> SchedulingDecision {
        if kernel.processes_blocked() {
            // No processes ready
            SchedulingDecision::TrySleep
        } else {
            let mut next = None; // This will be replaced, bc a process is guaranteed
                                 // to be ready if processes_blocked() is false

            // Find next ready process. Place any *empty* process slots, or not-ready
            // processes, at the back of the queue.
            for node in self.processes.iter() {
                match node.proc {
                    Some(proc) => {
                        if proc.ready() {
                            next = Some(proc.appid());
                            break;
                        }
                        self.processes.push_tail(self.processes.pop_head().unwrap());
                    }
                    None => {
                        self.processes.push_tail(self.processes.pop_head().unwrap());
                    }
                }
            }

            SchedulingDecision::RunProcess((next.unwrap(), None))
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
