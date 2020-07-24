//! Preemptive Priority Scheduler for Tock
//!
//! This scheduler allows for boards to set the priority of processes at boot,
//! and runs the highest priority process available at any point in time.
//! Kernel tasks (bottom half interrupt handling / deferred call handling)
//! always take priority over userspace processes.
//! Process priority is defined by the order the process appears in the PROCESSES
//! array. Notably, there is no need to enforce timeslices, as it is impossible
//! for a process running to not be the highest priority process at any point
//! without the process being descheduled (thanks to the check in leave_do_process()).

use crate::callback::AppId;
use crate::common::cells::OptionalCell;
use crate::common::dynamic_deferred_call::DynamicDeferredCall;
use crate::platform::Chip;
use crate::sched::{Kernel, Scheduler, SchedulingDecision, StoppedExecutingReason};

/// Preemptive Priority Scheduler
pub struct PrioritySched {
    kernel: &'static Kernel,
    running: OptionalCell<AppId>, // tracks currently executing process
}

impl PrioritySched {
    /// How long a process can run before being pre-empted
    pub const fn new(kernel: &'static Kernel) -> Self {
        Self {
            kernel,
            running: OptionalCell::empty(),
        }
    }
}

impl<C: Chip> Scheduler<C> for PrioritySched {
    fn next(&self, kernel: &Kernel) -> SchedulingDecision {
        if kernel.processes_blocked() {
            // No processes ready
            SchedulingDecision::Sleep
        } else {
            // Iterates in-order through the process array, always running
            // the first process it finds that is ready to run. This means
            // that processes with higher
            let next = self
                .kernel
                .get_process_iter()
                .find(|&proc| proc.ready())
                .map_or(None, |proc| Some(proc.appid()));
            self.running.insert(next);

            SchedulingDecision::RunProcess((next.unwrap(), None))
        }
    }

    unsafe fn continue_process(&self, _: AppId, chip: &C) -> bool {
        // In addition to checking for interrupts, also
        // checks if any higher priority processes have become ready.
        // This check is necessary because a system call by this process could make
        // another process ready, if this app is communicating
        // via IPC with a higher priority app
        chip.has_pending_interrupts()
            || DynamicDeferredCall::global_instance_calls_pending().unwrap_or(false)
            || self
                .kernel
                .get_process_iter()
                .position(|proc| proc.ready())
                .map_or(false, |ready_idx| {
                    self.running
                        .map_or(false, |running| ready_idx < running.index)
                })
    }

    fn result(&self, _: StoppedExecutingReason, _: Option<u32>) {
        self.running.clear()
    }
}
