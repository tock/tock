// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Implementation of [`ProcessRestart`] for use by capsules.
//!
//! Although the `ProcessRestartCapability` provides a finer grained
//! permission that only enables the ProcessManagement functionality
//! of restarting a process, restarting a process requires a
//! reference to the `Kernel` and possession of the
//! `ProcessManagementCapability`. The `ProcessRestart` capsule
//! implements the needed functionality to allow other processes
//! to restart a process (assuming the capsule possesses the
//! `ProcessRestartCapability`).

use kernel::capabilities::{ProcessManagementCapability, ProcessRestartCapability};
use kernel::process::{ProcessId, ProcessRestart};
use kernel::Kernel;

/// ProcessRestarter wraps the objects needed to restart a process.
///
/// Allows other capsules to restart processes without requiring
/// a reference to the `Kernel` and more powerful `ProcessManagementCapability`.
/// With the `ProcessRestarter` and the `ProcessRestartCapability`,
/// a capsule can restart any process .
pub struct ProcessRestarter<C: ProcessManagementCapability> {
    kernel: &'static Kernel,
    capability: C,
}

impl<C: ProcessManagementCapability> ProcessRestarter<C> {
    pub fn new(kernel: &'static Kernel, capability: C) -> Self {
        Self { kernel, capability }
    }
}

impl<C: ProcessManagementCapability> ProcessRestart for ProcessRestarter<C> {
    fn try_restart(&self, pid: ProcessId, _capability: &dyn ProcessRestartCapability) {
        self.kernel
            .process_each_capability(&self.capability, |process| {
                if process.processid() == pid {
                    kernel::debug!("Attempting to restart application with pid {:?}", pid);
                    process.try_restart(None);
                }
            })
    }
}
