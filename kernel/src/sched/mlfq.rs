//! Multilevel feedback queue scheduler for Tock
//!
//! Based on the MLFQ rules described in "Operating Systems: Three Easy Pieces"
//! By Remzi H. Arpaci-Dusseau and Andrea C. Arpaci-Dusseau
//!
//! This scheduler can be summarized by the following rules:
//!
//! Rule 1: If Priority(A) > Priority(B), and both are ready, A runs (B doesn’t).
//! Rule 2: If Priority(A) = Priority(B), A & B run in round-robin fashion using the
//!         time slice (quantum length) of the given queue.
//! Rule 3: When a job enters the system, it is placed at the highest priority (the topmost queue).
//! Rule 4: Once a job uses up its time allotment at a given level (regardless of how
//!         many times it has given up the CPU), its priority is reduced
//!         (i.e., it moves down one queue).
//! Rule 5: After some time period S, move all the jobs in the system to the topmost queue.

use crate::callback::AppId;
use crate::common::list::{List, ListLink, ListNode};
use crate::hil::time;
use crate::hil::time::Frequency;
use crate::platform::Chip;
use crate::sched::{Kernel, Scheduler, StoppedExecutingReason};
use core::cell::Cell;

#[derive(Default)]
struct MfProcState {
    /// Total CPU time used by this process while in current queue
    us_used_this_queue: Cell<u32>,
}

/// Nodes store per-process state
pub struct MLFQProcessNode<'a> {
    appid: AppId,
    state: MfProcState,
    next: ListLink<'a, MLFQProcessNode<'a>>,
}

impl<'a> MLFQProcessNode<'a> {
    pub fn new(appid: AppId) -> MLFQProcessNode<'a> {
        MLFQProcessNode {
            appid,
            state: MfProcState::default(),
            next: ListLink::empty(),
        }
    }
}

impl<'a> ListNode<'a, MLFQProcessNode<'a>> for MLFQProcessNode<'a> {
    fn next(&'a self) -> &'static ListLink<'a, MLFQProcessNode<'a>> {
        &self.next
    }
}

pub struct MLFQSched<'a, A: 'static + time::Alarm<'static>> {
    kernel: &'static Kernel,
    alarm: &'static A,
    pub processes: [List<'a, MLFQProcessNode<'a>>; 3], // Using Self::NUM_QUEUES causes rustc to crash..
    next_reset: Cell<u32>,
    last_timeslice: Cell<u32>,
    last_queue_idx: Cell<usize>,
}

impl<'a, A: 'static + time::Alarm<'static>> MLFQSched<'a, A> {
    /// How often to restore all processes to max priority
    pub const PRIORITY_REFRESH_PERIOD_MS: u32 = 5000;
    pub const NUM_QUEUES: usize = 3;

    pub fn new(kernel: &'static Kernel, alarm: &'static A) -> Self {
        Self {
            kernel,
            alarm,
            processes: [List::new(), List::new(), List::new()],
            next_reset: Cell::new(0),
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
        let mut first = true;
        for queue in self.processes.iter() {
            if first {
                continue;
            }
            first = false;
            match queue.pop_head() {
                Some(proc) => self.processes[0].push_tail(proc),
                None => continue,
            }
        }
    }

    /// Returns the process at the head of the highest priority queue containing a process
    /// that is ready to execute (as determined by `has_tasks()`)
    /// This method moves that node to the head of its queue.
    fn get_next_ready_process_node(&self) -> (Option<&MLFQProcessNode<'a>>, usize) {
        for (idx, queue) in self.processes.iter().enumerate() {
            let next = queue.iter().find(|node_ref| {
                self.kernel
                    .process_map_or(false, node_ref.appid, |proc| proc.ready())
            });
            if next.is_some() {
                // pop procs to back until we get to match
                loop {
                    let cur = queue.pop_head();
                    match cur {
                        Some(node) => {
                            if node as *const _ == next.unwrap() as *const _ {
                                queue.push_head(node);
                                // match! Put back on front
                                return (next, idx);
                            } else {
                                queue.push_tail(node);
                            }
                        }
                        None => {}
                    }
                }
            }
        }
        (None, 0)
    }
}

impl<'a, A: 'static + time::Alarm<'static>, C: Chip> Scheduler<C> for MLFQSched<'a, A> {
    fn next(&self) -> (Option<AppId>, Option<u32>) {
        let now = self.alarm.now();
        if now >= self.next_reset.get() {
            // Promote all processes to highest priority queue
            let delta = (Self::PRIORITY_REFRESH_PERIOD_MS * A::Frequency::frequency()) / 1000;
            self.next_reset.set(now.wrapping_add(delta));
            self.redeem_all_procs();
        }
        let (node_ref_opt, queue_idx) = self.get_next_ready_process_node();
        let node_ref = node_ref_opt.unwrap(); //Panic if fail bc processes_blocked()!
        let timeslice = self.get_timeslice_us(queue_idx) - node_ref.state.us_used_this_queue.get();
        let next = node_ref.appid;
        self.last_queue_idx.set(queue_idx);
        self.last_timeslice.set(timeslice);

        (Some(next), Some(timeslice))
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

    unsafe fn continue_process(&self, _: AppId, _: &C) -> bool {
        // This MLFQ scheduler only preempts processes if there is a timeslice expiration
        true
    }
}
