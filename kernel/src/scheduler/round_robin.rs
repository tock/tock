//! Round Robin Scheduler for Tock
//!
//! This scheduler is specifically a Round Robin Scheduler with Interrupts.
//!
//! See: <https://www.eecs.umich.edu/courses/eecs461/lecture/SWArchitecture.pdf>
//! for details.
//!
//! When hardware interrupts occur while a userspace process is executing, this
//! scheduler executes the top half of the interrupt, and then stops executing
//! the userspace process immediately and handles the bottom half of the
//! interrupt. This design decision was made to mimic the behavior of the
//! original Tock scheduler. In order to ensure fair use of timeslices, when
//! userspace processes are interrupted the scheduler timer is paused, and the
//! same process is resumed with the same scheduler timer value from when it was
//! interrupted.

use core::cell::Cell;
use core::marker::PhantomData;

use crate::collections::list::{ListNode, SinglyLinkedList};
use crate::kernel::{Kernel, StoppedExecutingReason};
use crate::platform::chip::Chip;
use crate::process::Process;
use crate::scheduler::{Scheduler, SchedulingDecision};

/// Round Robin Scheduler
pub struct RoundRobinSched<
    'a,
    N: 'a + ListNode<'a, Content = Option<&'static dyn Process>>,
    L: SinglyLinkedList<'a, N>,
> {
    time_remaining: Cell<u32>,
    pub processes: L,
    last_rescheduled: Cell<bool>,
    _node: PhantomData<&'a N>,
}

impl<
        'a,
        N: 'a + ListNode<'a, Content = Option<&'static dyn Process>>,
        L: SinglyLinkedList<'a, N>,
    > RoundRobinSched<'a, N, L>
{
    /// How long a process can run before being pre-empted
    const DEFAULT_TIMESLICE_US: u32 = 10000;
    pub const fn new(processes: L) -> RoundRobinSched<'a, N, L> {
        RoundRobinSched {
            processes,
            time_remaining: Cell::new(Self::DEFAULT_TIMESLICE_US),
            last_rescheduled: Cell::new(false),
            _node: PhantomData,
        }
    }
}

impl<
        'a,
        N: 'a + ListNode<'a, Content = Option<&'static dyn Process>>,
        L: SinglyLinkedList<'a, N>,
        C: Chip,
    > Scheduler<C> for RoundRobinSched<'a, N, L>
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
            let timeslice = if self.last_rescheduled.get() {
                self.time_remaining.get()
            } else {
                // grant a fresh timeslice
                self.time_remaining.set(Self::DEFAULT_TIMESLICE_US);
                Self::DEFAULT_TIMESLICE_US
            };
            assert!(timeslice != 0);

            SchedulingDecision::RunProcess((next.unwrap(), Some(timeslice)))
        }
    }

    fn result(&self, result: StoppedExecutingReason, execution_time_us: Option<u32>) {
        let execution_time_us = execution_time_us.unwrap(); // should never fail
        let reschedule = match result {
            StoppedExecutingReason::KernelPreemption => {
                if self.time_remaining.get() > execution_time_us {
                    self.time_remaining
                        .set(self.time_remaining.get() - execution_time_us);
                    true
                } else {
                    false
                }
            }
            _ => false,
        };
        self.last_rescheduled.set(reschedule);
        if !reschedule {
            self.processes.push_tail(self.processes.pop_head().unwrap());
        }
    }
}
