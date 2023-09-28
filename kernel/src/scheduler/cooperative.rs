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
use crate::process::Process;
use crate::process::StoppedExecutingReason;
use crate::scheduler::{Scheduler, SchedulingDecision};

/// A node in the linked list the scheduler uses to track processes
pub struct CoopProcessNode<'a> {
    proc: &'static Option<&'static dyn Process>,
    next: ListLink<'a, CoopProcessNode<'a>>,
}

impl<'a> CoopProcessNode<'a> {
    pub fn new(proc: &'static Option<&'static dyn Process>) -> CoopProcessNode<'a> {
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
    fn next(&self) -> SchedulingDecision {
        let mut first_head = None;
        let mut next = None;

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
            match node.proc {
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

        // next will not be None, because if we make a full iteration and nothing
        // is ready we return early
        SchedulingDecision::RunProcess((next.unwrap(), None))
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
