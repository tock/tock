// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Process-related policies in the Tock kernel.
//!
//! This file contains definitions of policies the Tock kernel can use when
//! managing processes. For example, these policies control decisions such as
//! whether a specific process should be restarted.

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
