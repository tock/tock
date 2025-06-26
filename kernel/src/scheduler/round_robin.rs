// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

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
use core::num::NonZeroU32;

use crate::collections::list::{List, ListLink, ListNode};
use crate::platform::chip::Chip;
use crate::process::ProcessSlot;
use crate::process::StoppedExecutingReason;
use crate::scheduler::{Scheduler, SchedulingDecision};

/// A node in the linked list the scheduler uses to track processes
/// Each node holds a pointer to a slot in the processes array
pub struct RoundRobinProcessNode<'a> {
    proc: &'static ProcessSlot,
    next: ListLink<'a, RoundRobinProcessNode<'a>>,
}

impl<'a> RoundRobinProcessNode<'a> {
    pub const fn new(proc: &'static ProcessSlot) -> RoundRobinProcessNode<'a> {
        RoundRobinProcessNode {
            proc,
            next: ListLink::empty(),
        }
    }
}

impl<'a> ListNode<'a, RoundRobinProcessNode<'a>> for RoundRobinProcessNode<'a> {
    fn next(&'a self) -> &'a ListLink<'a, RoundRobinProcessNode<'a>> {
        &self.next
    }
}

/// Round Robin Scheduler
pub struct RoundRobinSched<'a> {
    time_remaining: Cell<u32>,
    timeslice_length: u32,
    pub processes: List<'a, RoundRobinProcessNode<'a>>,
    last_rescheduled: Cell<bool>,
}

impl<'a> RoundRobinSched<'a> {
    /// How long a process can run before being pre-empted
    const DEFAULT_TIMESLICE_US: u32 = 10000;
    pub const fn new() -> RoundRobinSched<'a> {
        Self::new_with_time(Self::DEFAULT_TIMESLICE_US)
    }

    pub const fn new_with_time(time_us: u32) -> RoundRobinSched<'a> {
        RoundRobinSched {
            time_remaining: Cell::new(time_us),
            timeslice_length: time_us,
            processes: List::new(),
            last_rescheduled: Cell::new(false),
        }
    }
}

impl<C: Chip> Scheduler<C> for RoundRobinSched<'_> {
    fn next(&self) -> SchedulingDecision {
        let mut first_head = None;
        let mut next = None;

        // Find the first ready process in the queue. Place any *empty* process slots,
        // or not-ready processes, at the back of the queue.
        while let Some(node) = self.processes.head() {
            // Ensure we do not loop forever if all processes are not ready
            match first_head {
                None => first_head = Some(node),
                Some(first_head) => {
                    // We made a full iteration and nothing was ready. Try to sleep instead
                    if core::ptr::eq(first_head, node) {
                        return SchedulingDecision::TrySleep;
                    }
                }
            }
            match node.proc.get() {
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

        let next = match next {
            Some(p) => p,
            None => {
                // No processes on the system
                return SchedulingDecision::TrySleep;
            }
        };

        let timeslice = if self.last_rescheduled.get() {
            self.time_remaining.get()
        } else {
            // grant a fresh timeslice
            self.time_remaining.set(self.timeslice_length);
            self.timeslice_length
        };
        // Why should this panic?
        let non_zero_timeslice = NonZeroU32::new(timeslice).unwrap();

        SchedulingDecision::RunProcess((next, Some(non_zero_timeslice)))
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
