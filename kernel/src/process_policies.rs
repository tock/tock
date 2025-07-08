// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Process-related policies in the Tock kernel.
//!
//! This file contains definitions of policies the Tock kernel can use when
//! managing processes. For example, these policies control decisions such as
//! whether a specific process should be restarted.

use crate::platform::chip::Chip;
use crate::process;
use crate::process::Process;
use crate::process_standard::ProcessStandard;
use crate::process_standard::ProcessStandardDebug;
use crate::storage_permissions::StoragePermissions;

/// Generic trait for implementing a policy on what to do when a process faults.
///
/// Implementations can use the `Process` reference to decide which action to
/// take. Implementations can also use `debug!()` to print messages if desired.
pub trait ProcessFaultPolicy {
    /// Decide which action the kernel should take in response to `process`
    /// faulting.
    fn action(&self, process: &dyn Process) -> process::FaultAction;
}

/// Generic trait for implementing a policy on how applications should be
/// assigned storage permissions.
pub trait ProcessStandardStoragePermissionsPolicy<C: Chip, D: ProcessStandardDebug> {
    /// Return the storage permissions for the specified `process`.
    fn get_permissions(&self, process: &ProcessStandard<C, D>) -> StoragePermissions;
}

// Any platforms that do not issue storage permissions can use `&()` as the
// [`ProcessStandardStoragePermissionsPolicy`]. This will only provide null
// permissions (that is, no permission to access persistent storage).
impl<C: Chip, D: ProcessStandardDebug> ProcessStandardStoragePermissionsPolicy<C, D> for () {
    fn get_permissions(&self, _process: &ProcessStandard<C, D>) -> StoragePermissions {
        StoragePermissions::new_null()
    }
}
