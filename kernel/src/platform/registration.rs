// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use crate::errorcode::ErrorCode;
use crate::process::ProcessId;

/// Filter policy abstraction for userspace registration interfaces.
///
/// This trait enables capsules that permit registrations from userspace
/// processes to support customized registration filters. Kernels can configure
/// the registration policy of registration-based capsules by implementing this
/// trait with the desired registration policy.
///
/// # Use Cases
///
/// The canonical use case is userspace processes registering some service with
/// the kernel. For example, a process may provide a service via IPC, and
/// registers that service with the kernel. This filter allows the kernel to
/// decide whether to permit that registration.
///
pub trait RegistrationFilter {
    /// The type of registration identifier specific to the registration
    /// interface.
    ///
    /// Different registration interfaces will use different identifiers to
    /// identify what type of service is being registered. For example, services
    /// could be identified by a string name (`&str`) or a well-known id
    /// (`usize`).
    type RegistrationIdentifier;

    /// Called to determine if a registration request should be permitted.
    ///
    /// A registration interface can use this function to determine whether to
    /// permit a userspace process to proceed with the registration.
    ///
    /// # Return
    ///
    /// To permit the registration, return `Ok(())`.
    ///
    /// To deny the registration, return `Err()` with an [`ErrorCode`]. Using
    /// [`ErrorCode::FAIL`] is recommended.
    fn filter_registration(
        &self,
        processid: ProcessId,
        registrationid: &Self::RegistrationIdentifier,
    ) -> Result<(), ErrorCode>;
}
