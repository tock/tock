// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Fixed Priority Scheduler for Tock
//!
//! This scheduler assigns priority to processes based on their order in the
//! `PROCESSES` array, and runs the highest priority process available at any
//! point in time. Kernel tasks (bottom half interrupt handling / deferred call
//! handling) always take priority over userspace processes.
//!
//! Notably, there is no need to enforce timeslices, as it is impossible for a
//! process running to not be the highest priority process at any point while it
//! is running. The only way for a process to longer be the highest priority is
//! for an interrupt to occur, which will cause the process to stop running.

use kernel::capabilities::ProcessManagementCapability;
use kernel::deferred_call::DeferredCall;
use kernel::platform::chip::Chip;
use kernel::process::ProcessId;
use kernel::process::StoppedExecutingReason;
use kernel::scheduler::{Scheduler, SchedulingDecision};
use kernel::utilities::cells::OptionalCell;
use kernel::Kernel;

/// Priority scheduler based on the order of processes in the `PROCESSES` array.
pub struct PrioritySched<CAP: ProcessManagementCapability> {
    kernel: &'static Kernel,
    running: OptionalCell<(usize, ProcessId)>,
    cap: CAP,
}

impl<CAP: ProcessManagementCapability> PrioritySched<CAP> {
    pub const fn new(kernel: &'static Kernel, cap: CAP) -> Self {
        Self {
            kernel,
            running: OptionalCell::empty(),
            cap,
        }
    }
}

impl<C: Chip, CAP: ProcessManagementCapability> Scheduler<C> for PrioritySched<CAP> {
    fn next(&self) -> SchedulingDecision {
        // Iterates in-order through the process array, always running the
        // first process it finds that is ready to run. This enforces the
        // priorities of all processes.
        let next = self
            .kernel
            .process_iter_capability(&self.cap)
            .enumerate()
            .find(|(_i, proc)| proc.ready())
            .map(|(i, proc)| (i, proc.processid()));
        self.running.insert(next);

        next.map_or(SchedulingDecision::TrySleep, |next| {
            SchedulingDecision::RunProcess((next.1, None))
        })
    }

    fn continue_process(&self, _: ProcessId, chip: &C) -> bool {
        // In addition to checking for interrupts, also checks if any higher
        // priority processes have become ready. This check is necessary because
        // a system call by this process could make another process ready, if
        // this app is communicating via IPC with a higher priority app.
        !(chip.has_pending_interrupts()
            || DeferredCall::has_tasks()
            || self
                .kernel
                .process_iter_capability(&self.cap)
                .enumerate()
                .find(|(_i, proc)| proc.ready())
                .is_some_and(|(i, _ready_proc)| {
                    self.running.map_or(false, |running| i < running.0)
                }))
    }

    fn result(&self, _: StoppedExecutingReason, _: Option<u32>) {
        self.running.clear()
    }
}
