// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Multilevel feedback queue scheduler for Tock
//!
//! Based on the MLFQ rules described in "Operating Systems: Three Easy Pieces"
//! by Remzi H. Arpaci-Dusseau and Andrea C. Arpaci-Dusseau.
//!
//! This scheduler can be summarized by the following rules:
//!
//! - Rule 1: If Priority(A) > Priority(B), and both are ready, A runs (B
//!   doesn't).
//! - Rule 2: If Priority(A) = Priority(B), A & B run in round-robin fashion
//!   using the time slice (quantum length) of the given queue.
//! - Rule 3: When a job enters the system, it is placed at the highest priority
//!   (the topmost queue).
//! - Rule 4: Once a job uses up its time allotment at a given level (regardless
//!   of how many times it has given up the CPU), its priority is reduced (i.e.,
//!   it moves down one queue).
//! - Rule 5: After some time period S, move all the jobs in the system to the
//!   topmost queue.

use core::cell::Cell;
use core::num::NonZeroU32;

use crate::collections::list::{List, ListLink, ListNode};
use crate::hil::time::{self, ConvertTicks, Ticks};
use crate::platform::chip::Chip;
use crate::process::Process;
use crate::process::StoppedExecutingReason;
use crate::scheduler::{Scheduler, SchedulingDecision};

#[derive(Default)]
struct MfProcState {
    /// Total CPU time used by this process while in current queue
    us_used_this_queue: Cell<u32>,
}

/// Nodes store per-process state
pub struct MLFQProcessNode<'a> {
    proc: &'static Option<&'static dyn Process>,
    state: MfProcState,
    next: ListLink<'a, MLFQProcessNode<'a>>,
}

impl<'a> MLFQProcessNode<'a> {
    pub fn new(proc: &'static Option<&'static dyn Process>) -> MLFQProcessNode<'a> {
        MLFQProcessNode {
            proc,
            state: MfProcState::default(),
            next: ListLink::empty(),
        }
    }
}

impl<'a> ListNode<'a, MLFQProcessNode<'a>> for MLFQProcessNode<'a> {
    fn next(&'a self) -> &'a ListLink<'a, MLFQProcessNode<'a>> {
        &self.next
    }
}

pub struct MLFQSched<'a, A: 'static + time::Alarm<'static>> {
    alarm: &'static A,
    pub processes: [List<'a, MLFQProcessNode<'a>>; 3], // Using Self::NUM_QUEUES causes rustc to crash..
    next_reset: Cell<A::Ticks>,
    last_reset_check: Cell<A::Ticks>,
    last_timeslice: Cell<u32>,
    last_queue_idx: Cell<usize>,
}

impl<'a, A: 'static + time::Alarm<'static>> MLFQSched<'a, A> {
    /// How often to restore all processes to max priority
    pub const PRIORITY_REFRESH_PERIOD_MS: u32 = 5000;
    pub const NUM_QUEUES: usize = 3;

    pub fn new(alarm: &'static A) -> Self {
        Self {
            alarm,
            processes: [List::new(), List::new(), List::new()],
            next_reset: Cell::new(A::Ticks::from(0)),
            last_reset_check: Cell::new(A::Ticks::from(0)),
            last_timeslice: Cell::new(0),
            last_queue_idx: Cell::new(0),
        }
    }

    fn get_timeslice_us(&self, queue_idx: usize) -> u32 {
        match queue_idx {
            0 => 10000,
            1 => 20000,
            2 => 50000,
            _ => panic!("invalid queue idx"),
        }
    }

    fn redeem_all_procs(&self) {
        for queue in self.processes.iter().skip(1) {
            if let Some(proc) = queue.pop_head() {
                self.processes[0].push_tail(proc)
            }
        }
    }

    /// Returns the process at the head of the highest priority queue containing a process
    /// that is ready to execute (as determined by `has_tasks()`)
    /// This method moves that node to the head of its queue.
    fn get_next_ready_process_node(&self) -> (Option<&MLFQProcessNode<'a>>, usize) {
        for (idx, queue) in self.processes.iter().enumerate() {
            let next = queue
                .iter()
                .find(|node_ref| node_ref.proc.is_some_and(|proc| proc.ready()));
            if next.is_some() {
                // pop procs to back until we get to match
                loop {
                    let cur = queue.pop_head();
                    if let Some(node) = cur {
                        if core::ptr::eq(node, next.unwrap()) {
                            queue.push_head(node);
                            // match! Put back on front
                            return (next, idx);
                        } else {
                            queue.push_tail(node);
                        }
                    }
                }
            }
        }
        (None, 0)
    }
}

impl<A: 'static + time::Alarm<'static>, C: Chip> Scheduler<C> for MLFQSched<'_, A> {
    fn next(&self) -> SchedulingDecision {
        let now = self.alarm.now();
        let next_reset = self.next_reset.get();
        let last_reset_check = self.last_reset_check.get();

        // storing last reset check is necessary to avoid missing a reset when the underlying
        // alarm wraps around
        if !now.within_range(last_reset_check, next_reset) {
            // Promote all processes to highest priority queue
            self.next_reset
                .set(now.wrapping_add(self.alarm.ticks_from_ms(Self::PRIORITY_REFRESH_PERIOD_MS)));
            self.redeem_all_procs();
        }
        self.last_reset_check.set(now);
        let (node_ref_opt, queue_idx) = self.get_next_ready_process_node();
        if node_ref_opt.is_none() {
            return SchedulingDecision::TrySleep;
        }
        let node_ref = node_ref_opt.unwrap();
        let timeslice = self.get_timeslice_us(queue_idx) - node_ref.state.us_used_this_queue.get();
        let next = node_ref.proc.unwrap().processid();
        self.last_queue_idx.set(queue_idx);
        self.last_timeslice.set(timeslice);

        SchedulingDecision::RunProcess((next, NonZeroU32::new(timeslice)))
    }

    fn result(&self, result: StoppedExecutingReason, execution_time_us: Option<u32>) {
        let execution_time_us = execution_time_us.unwrap(); // should never fail as we never run cooperatively
        let queue_idx = self.last_queue_idx.get();
        // Last executed node will always be at head of its queue
        let node_ref = self.processes[queue_idx].head().unwrap();
        node_ref
            .state
            .us_used_this_queue
            .set(self.last_timeslice.get() - execution_time_us);

        let punish = result == StoppedExecutingReason::TimesliceExpired;
        if punish {
            node_ref.state.us_used_this_queue.set(0);
            let next_queue = if queue_idx == Self::NUM_QUEUES - 1 {
                queue_idx
            } else {
                queue_idx + 1
            };
            self.processes[next_queue].push_tail(self.processes[queue_idx].pop_head().unwrap());
        } else {
            self.processes[queue_idx].push_tail(self.processes[queue_idx].pop_head().unwrap());
        }
    }
}
