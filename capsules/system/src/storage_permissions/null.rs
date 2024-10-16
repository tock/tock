// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

use kernel::platform::chip::Chip;
use kernel::storage_permissions::StoragePermissions;

/// Always assign no storage permissions.
pub struct NullStoragePermissions<C: Chip, D: kernel::process::ProcessStandardDebug> {
    _chip: core::marker::PhantomData<C>,
    _debug: core::marker::PhantomData<D>,
}

impl<C: Chip, D: kernel::process::ProcessStandardDebug> NullStoragePermissions<C, D> {
    pub fn new() -> Self {
        Self {
            _chip: core::marker::PhantomData,
            _debug: core::marker::PhantomData,
        }
    }
}

impl<C: Chip, D: kernel::process::ProcessStandardDebug>
    kernel::process::ProcessStandardStoragePermissionsPolicy<C, D>
    for NullStoragePermissions<C, D>
{
    fn get_permissions(
        &self,
        _process: &kernel::process::ProcessStandard<C, D>,
    ) -> StoragePermissions {
        StoragePermissions::new_null()
    }
}
