// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

use kernel::platform::chip::Chip;
use kernel::storage_permissions::StoragePermissions;

/// Always assign no storage permissions.
pub struct NullStoragePermissions<C: Chip> {
    _chip: core::marker::PhantomData<C>,
}

impl<C: Chip> NullStoragePermissions<C> {
    pub fn new() -> Self {
        Self {
            _chip: core::marker::PhantomData,
        }
    }
}

impl<C: Chip> kernel::process::ProcessStandardStoragePermissionsPolicy<C>
    for NullStoragePermissions<C>
{
    fn get_permissions(
        &self,
        _process: &kernel::process::ProcessStandard<C>,
    ) -> StoragePermissions {
        StoragePermissions::new_null()
    }
}
