//! Round Robin Scheduler for Tock
//! This scheduler is specifically a Round Robin Scheduler with Interrupts.
//!
//! See: https://www.eecs.umich.edu/courses/eecs461/lecture/SWArchitecture.pdf
//! for details.
//! When hardware interrupts occur while a userspace process is executing,
//! this scheduler executes the top half of the interrupt,
//! and then stops executing the userspace process immediately and handles the bottom
//! half of the interrupt. This design decision was made to mimic the behavior of the
//! original Tock scheduler. In order to ensure fair use of timeslices, when
//! userspace processes are interrupted the systick is paused, and the same process
//! is resumed with the same systick value from when it was interrupted.

use crate::callback::AppId;
use crate::common::list::{List, ListLink, ListNode};
use crate::platform::Chip;
use crate::sched::{Kernel, Scheduler, StoppedExecutingReason};
use core::cell::Cell;

/// A node in the linked list the scheduler uses to track processes
pub struct RoundRobinProcessNode<'a> {
    appid: AppId,
    next: ListLink<'a, RoundRobinProcessNode<'a>>,
}

impl<'a> RoundRobinProcessNode<'a> {
    pub fn new(appid: AppId) -> RoundRobinProcessNode<'a> {
        RoundRobinProcessNode {
            appid,
            next: ListLink::empty(),
        }
    }
}

impl<'a> ListNode<'a, RoundRobinProcessNode<'a>> for RoundRobinProcessNode<'a> {
    fn next(&'a self) -> &'a ListLink<'a, RoundRobinProcessNode> {
        &self.next
    }
}

/// Round Robin Scheduler
pub struct RoundRobinSched<'a> {
    time_remaining: Cell<u32>,
    pub processes: List<'a, RoundRobinProcessNode<'a>>,
    last_rescheduled: Cell<bool>,
}

impl<'a> RoundRobinSched<'a> {
    /// How long a process can run before being pre-empted
    const DEFAULT_TIMESLICE_US: u32 = 10000;
    pub const fn new() -> RoundRobinSched<'a> {
        RoundRobinSched {
            time_remaining: Cell::new(Self::DEFAULT_TIMESLICE_US),
            processes: List::new(),
            last_rescheduled: Cell::new(false),
        }
    }
}

impl<'a, C: Chip> Scheduler<C> for RoundRobinSched<'a> {
    fn next(&self, kernel: &Kernel) -> (Option<AppId>, Option<u32>) {
        if kernel.processes_blocked() {
            (None, None)
        } else {
            let next = self.processes.head().unwrap().appid;
            let timeslice = if self.last_rescheduled.get() {
                self.time_remaining.get()
            } else {
                Self::DEFAULT_TIMESLICE_US
            };
            assert!(timeslice != 0);

            (Some(next), Some(timeslice))
        }
    }

    fn result(&self, result: StoppedExecutingReason, execution_time_us: Option<u32>) {
        let execution_time_us = execution_time_us.unwrap(); // should never fail
        self.time_remaining
            .set(self.time_remaining.get() - execution_time_us);
        let reschedule = match result {
            StoppedExecutingReason::KernelPreemption => true,
            _ => false,
        };
        self.last_rescheduled.set(reschedule);
        if !reschedule {
            self.processes.push_tail(self.processes.pop_head().unwrap());
        }
    }
}
