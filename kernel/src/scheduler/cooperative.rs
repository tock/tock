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

use core::marker::PhantomData;

use crate::collections::list::{ListNode, SinglyLinkedList};
use crate::kernel::{Kernel, StoppedExecutingReason};
use crate::platform::chip::Chip;
use crate::process::Process;
use crate::scheduler::{Scheduler, SchedulingDecision};

/// Cooperative Scheduler
pub struct CooperativeSched<
    'a,
    N: 'a + ListNode<'a, Content = Option<&'static dyn Process>>,
    L: SinglyLinkedList<'a, N>,
> {
    pub processes: L,
    _node: PhantomData<&'a N>,
}

impl<
        'a,
        N: 'a + ListNode<'a, Content = Option<&'static dyn Process>>,
        L: SinglyLinkedList<'a, N>,
    > CooperativeSched<'a, N, L>
{
    pub const fn new(processes: L) -> CooperativeSched<'a, N, L> {
        CooperativeSched {
            processes,
            _node: PhantomData,
        }
    }
}

impl<
        'a,
        C: Chip,
        N: 'a + ListNode<'a, Content = Option<&'static dyn Process>>,
        L: SinglyLinkedList<'a, N>,
    > Scheduler<C> for CooperativeSched<'a, N, L>
{
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
                match node {
                    Some(proc) => {
                        if proc.ready() {
                            next = Some(proc.processid());
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
