// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Process-related policies in the Tock kernel.
//!
//! This file contains definitions and implementations of policies the Tock
//! kernel can use when managing processes. For example, these policies control
//! decisions such as whether a specific process should be restarted.

use crate::process;
use crate::process::Process;

/// Generic trait for implementing a policy on what to do when a process faults.
///
/// Implementations can use the `Process` reference to decide which action to
/// take. Implementations can also use `debug!()` to print messages if desired.
pub trait ProcessFaultPolicy {
    /// Decide which action the kernel should take in response to `process`
    /// faulting.
    fn action(&self, process: &dyn Process) -> process::FaultAction;
}

/// Simply panic the entire board if a process faults.
pub struct PanicFaultPolicy {}

impl ProcessFaultPolicy for PanicFaultPolicy {
    fn action(&self, _: &dyn Process) -> process::FaultAction {
        process::FaultAction::Panic
    }
}

/// Simply stop the process and no longer schedule it if a process faults.
pub struct StopFaultPolicy {}

impl ProcessFaultPolicy for StopFaultPolicy {
    fn action(&self, _: &dyn Process) -> process::FaultAction {
        process::FaultAction::Stop
    }
}

/// Stop the process and no longer schedule it if a process faults, but also
/// print a debug message notifying the user that the process faulted and
/// stopped.
pub struct StopWithDebugFaultPolicy {}

impl ProcessFaultPolicy for StopWithDebugFaultPolicy {
    fn action(&self, process: &dyn Process) -> process::FaultAction {
        crate::debug!(
            "Process {} faulted and was stopped.",
            process.get_process_name()
        );
        process::FaultAction::Stop
    }
}

/// Always restart the process if it faults.
pub struct RestartFaultPolicy {}

impl ProcessFaultPolicy for RestartFaultPolicy {
    fn action(&self, _: &dyn Process) -> process::FaultAction {
        process::FaultAction::Restart
    }
}

/// Implementation of `ProcessFaultPolicy` that uses a threshold to decide
/// whether to restart a process when it faults. If the process has been
/// restarted more times than the threshold then the process will be stopped
/// and no longer scheduled.
pub struct ThresholdRestartFaultPolicy {
    threshold: usize,
}

impl ThresholdRestartFaultPolicy {
    pub const fn new(threshold: usize) -> ThresholdRestartFaultPolicy {
        ThresholdRestartFaultPolicy { threshold }
    }
}

impl ProcessFaultPolicy for ThresholdRestartFaultPolicy {
    fn action(&self, process: &dyn Process) -> process::FaultAction {
        if process.get_restart_count() <= self.threshold {
            process::FaultAction::Restart
        } else {
            process::FaultAction::Stop
        }
    }
}

/// Implementation of `ProcessFaultPolicy` that uses a threshold to decide
/// whether to restart a process when it faults. If the process has been
/// restarted more times than the threshold then the board will panic.
pub struct ThresholdRestartThenPanicFaultPolicy {
    threshold: usize,
}

impl ThresholdRestartThenPanicFaultPolicy {
    pub const fn new(threshold: usize) -> ThresholdRestartThenPanicFaultPolicy {
        ThresholdRestartThenPanicFaultPolicy { threshold }
    }
}

impl ProcessFaultPolicy for ThresholdRestartThenPanicFaultPolicy {
    fn action(&self, process: &dyn Process) -> process::FaultAction {
        if process.get_restart_count() <= self.threshold {
            process::FaultAction::Restart
        } else {
            process::FaultAction::Panic
        }
    }
}
