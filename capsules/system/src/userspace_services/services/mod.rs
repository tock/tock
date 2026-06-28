// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Userspace service service interfaces.
//!
//! Service interfaces bridge a HIL client to the userspace service application through the registry,
//! preventing the HIL client from needing code to interact with the registry.
//! This being the case,
//! much of the implementation of the service interface is dedicated
//! to mapping HIL trait functions to [`usercall()`](super::usercall::UserspaceServiceAccess::usercall()) calls.

pub mod digest;

/// Service function identifier.
///
/// Identifies the userspace service by its function,
/// corresponding to the HIL the interface implements.
#[derive(Debug, PartialEq)]
pub enum Role {
    Digest = 0x11,
}
