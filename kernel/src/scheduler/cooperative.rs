// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

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

use crate::collections::list::{List, ListLink, ListNode};
use crate::platform::chip::Chip;
use crate::process::ProcessSlot;
use crate::process::StoppedExecutingReason;
use crate::scheduler::{Scheduler, SchedulingDecision};

/// A node in the linked list the scheduler uses to track processes
pub struct CoopProcessNode<'a> {
    proc: &'static ProcessSlot,
    next: ListLink<'a, CoopProcessNode<'a>>,
}

impl<'a> CoopProcessNode<'a> {
    pub fn new(proc: &'static ProcessSlot) -> CoopProcessNode<'a> {
        CoopProcessNode {
            proc,
            next: ListLink::empty(),
        }
    }
}

impl<'a> ListNode<'a, CoopProcessNode<'a>> for CoopProcessNode<'a> {
    fn next(&'a self) -> &'a ListLink<'a, CoopProcessNode<'a>> {
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

impl<C: Chip> Scheduler<C> for CooperativeSched<'_> {
    fn next(&self) -> SchedulingDecision {
        let mut first_head = None;

        // Find the first ready process in the queue. Place any *empty* process slots,
        // or not-ready processes, at the back of the queue.
        while let Some(node) = self.processes.head() {
            // Ensure we do not loop forever if all processes are not not ready
            match first_head {
                None => first_head = Some(node),
                Some(first_head) => {
                    // We make a full iteration and nothing was ready. Try to sleep instead
                    if core::ptr::eq(first_head, node) {
                        return SchedulingDecision::TrySleep;
                    }
                }
            }
            match node.proc.get() {
                Some(proc) => {
                    if proc.ready() {
                        let next = proc.processid();
                        return SchedulingDecision::RunProcess((next, None));
                    }
                    self.processes.push_tail(self.processes.pop_head().unwrap());
                }
                None => {
                    self.processes.push_tail(self.processes.pop_head().unwrap());
                }
            }
        }

        // If the length of `self.processes` is 0, the while loop never executes. In this case,
        // return `SchedulingDecision::TrySleep` as there is no process that can be scheduled.
        SchedulingDecision::TrySleep
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
