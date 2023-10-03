// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Interface for Tock kernel schedulers.

pub mod cooperative;
pub mod mlfq;
pub mod priority;
pub mod round_robin;

use crate::deferred_call::DeferredCall;
use crate::platform::chip::Chip;
use crate::process::ProcessId;
use crate::process::StoppedExecutingReason;

/// Trait which any scheduler must implement.
pub trait Scheduler<C: Chip> {
    /// Decide the next kernel operation.
    ///
    /// The scheduler must decide if the kernel:
    ///
    /// * has to do work
    /// * run a process. An identifier for the process and its assigned time slice in microseconds
    /// must be provided. If no time slice is specified, the process will run cooperatively.
    /// * try to enter in sleep mode
    fn next(&self, chip: &C) -> SchedulingDecision;

    /// Inform the scheduler of why the last process stopped executing, and how
    /// long it executed for. Notably, `execution_time_us` will be `None`
    /// if the the scheduler requested this process be run cooperatively.
    fn result(&self, result: StoppedExecutingReason, execution_time_us: Option<u32>);

    /// Ask the scheduler whether to take a break from executing userspace
    /// processes to handle kernel tasks. Most schedulers will use this default
    /// implementation, which always prioritizes kernel work, but schedulers
    /// that wish to defer interrupt handling may reimplement it.
    fn should_kernel_do_work(&self, chip: &C) -> bool {
        chip.has_pending_interrupts() || DeferredCall::has_tasks()
    }

    /// Ask the scheduler whether to continue trying to execute a process.
    ///
    /// Once a process is scheduled the kernel will try to execute it until it
    /// has no more work to do or exhausts its timeslice. The kernel will call
    /// this function before every loop to check with the scheduler if it wants
    /// to continue trying to execute this process.
    ///
    /// Most schedulers will use this default implementation, which causes the
    /// `do_process()` loop to return if there are interrupts or deferred calls
    /// that need to be serviced. However, schedulers which wish to defer
    /// interrupt handling may change this, or priority schedulers which wish to
    /// check if the execution of the current process has caused a higher
    /// priority process to become ready (such as in the case of IPC). If this
    /// returns `false`, then `do_process` will exit with a `KernelPreemption`.
    ///
    /// `id` is the identifier of the currently active process.
    unsafe fn continue_process(&self, _id: ProcessId, chip: &C) -> bool {
        !(chip.has_pending_interrupts() || DeferredCall::has_tasks())
    }
}

/// Enum representing the actions the scheduler can request in each call to
/// `scheduler.next()`.
#[derive(Copy, Clone)]
pub enum SchedulingDecision {
    /// Tell the kernel to do work, such as interrupts or deferred calls
    KernelWork,

    /// Tell the kernel to run the specified process with the passed timeslice.
    /// If `None` is passed as a timeslice, the process will be run
    /// cooperatively.
    RunProcess((ProcessId, Option<u32>)),

    /// Tell the kernel to go to sleep. Notably, if the scheduler asks the
    /// kernel to sleep when kernel tasks are ready, the kernel will not sleep,
    /// and will instead restart the main loop and call `next()` again.
    TrySleep,
}
