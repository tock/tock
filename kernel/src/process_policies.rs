//! Process-related policies in the Tock kernel.
//!
//! This file contains definitions and implementations of policies the Tock
//! kernel can use when managing processes. For example, these policies control
//! decisions such as whether a specific process should be restarted.

use crate::process::Process;

/// Generic trait for implementing process restart policies.
///
/// This policy allows a board to specify how the kernel should decide whether
/// to restart an app after it crashes.
pub trait ProcessRestartPolicy {
    /// Decide whether to restart the `process` or not.
    ///
    /// Returns `true` if the process should be restarted, `false` otherwise.
    fn should_restart(&self, process: &dyn Process) -> bool;
}

/// Implementation of `ProcessRestartPolicy` that uses a threshold to decide
/// whether to restart an app. If the app has been restarted more times than the
/// threshold then the app will no longer be restarted.
pub struct ThresholdRestart {
    threshold: usize,
}

impl ThresholdRestart {
    pub const fn new(threshold: usize) -> ThresholdRestart {
        ThresholdRestart { threshold }
    }
}

impl ProcessRestartPolicy for ThresholdRestart {
    fn should_restart(&self, process: &dyn Process) -> bool {
        process.get_restart_count() <= self.threshold
    }
}

/// Implementation of `ProcessRestartPolicy` that uses a threshold to decide
/// whether to restart an app. If the app has been restarted more times than the
/// threshold then the system will panic.
pub struct ThresholdRestartThenPanic {
    threshold: usize,
}

impl ThresholdRestartThenPanic {
    pub const fn new(threshold: usize) -> ThresholdRestartThenPanic {
        ThresholdRestartThenPanic { threshold }
    }
}

impl ProcessRestartPolicy for ThresholdRestartThenPanic {
    fn should_restart(&self, process: &dyn Process) -> bool {
        if process.get_restart_count() <= self.threshold {
            true
        } else {
            panic!("Restart threshold surpassed!");
        }
    }
}

/// Implementation of `ProcessRestartPolicy` that unconditionally restarts the
/// app.
pub struct AlwaysRestart {}

impl AlwaysRestart {
    pub const fn new() -> AlwaysRestart {
        AlwaysRestart {}
    }
}

impl ProcessRestartPolicy for AlwaysRestart {
    fn should_restart(&self, _process: &dyn Process) -> bool {
        true
    }
}
