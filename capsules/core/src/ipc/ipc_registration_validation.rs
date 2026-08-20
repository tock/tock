// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Validation function type for IPC registration validation.
//!
//! This function is used to synchronously validate a registration attempt
//! before allowing it. The implementation can choose to allow or deny
//! registration based on the process and name provided.

use kernel::{ErrorCode, ProcessId};

/// Validation function signature
///
/// Arguments:
///  * ProcessId to be validated
///  * Name it is attempting to register with
///
/// Return:
///  * Result. To allow registration, return `Ok(())`. To deny registration
///    return `Err()` with an [`ErrorCode`]. Using [`ErrorCode::FAIL`] is
///    recommended.
pub type IpcRegistrationValidationFunction = fn(ProcessId, &[u8]) -> Result<(), ErrorCode>;
