// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use kernel::errorcode::ErrorCode;
use kernel::platform::registration::RegistrationFilter;
use kernel::process::ShortId;

/// Null filter for [`IpcRegistryStringName`].
///
/// This permits all registrations from all processes.
pub struct IpcStringNameRegistrationFilterNull;

impl RegistrationFilter for IpcStringNameRegistrationFilterNull {
    type RegistrationIdentifier = [u8; super::ipc_registry_string_name::MAX_STRING_LEN];
    fn filter_registration(
        &self,
        _appid: ShortId,
        _registrationid: &Self::RegistrationIdentifier,
    ) -> Result<(), ErrorCode> {
        Ok(())
    }
}
