// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

use capsules_system::storage_permissions::tbf_header::TbfHeaderStoragePermissions;
use kernel::capabilities::ApplicationStorageCapability;
use kernel::platform::chip::Chip;
use kernel::process::Process;
use kernel::process::ShortId;
use kernel::storage_permissions::StoragePermissions;

/// Assign storage permissions from the TBF header if they exist, or default to
/// accessing own state.
pub struct InvsStoragePermissions<
    C: Chip,
    D: kernel::process::ProcessStandardDebug,
    CAP: ApplicationStorageCapability + Clone,
> {
    tbf_permissions: TbfHeaderStoragePermissions<C, D, CAP>,
    cap: CAP,
    _chip: core::marker::PhantomData<C>,
    _debug: core::marker::PhantomData<D>,
}

impl<
        C: Chip,
        D: kernel::process::ProcessStandardDebug,
        CAP: ApplicationStorageCapability + Clone,
    > InvsStoragePermissions<C, D, CAP>
{
    pub fn new(cap: CAP) -> Self {
        Self {
            tbf_permissions: TbfHeaderStoragePermissions::new(cap.clone()),
            cap,
            _chip: core::marker::PhantomData,
            _debug: core::marker::PhantomData,
        }
    }
}

impl<
        C: Chip,
        D: kernel::process::ProcessStandardDebug,
        CAP: ApplicationStorageCapability + Clone,
    > kernel::process::ProcessStandardStoragePermissionsPolicy<C, D>
    for InvsStoragePermissions<C, D, CAP>
{
    fn get_permissions(
        &self,
        process: &kernel::process::ProcessStandard<C, D>,
    ) -> StoragePermissions {
        // If we have a fixed ShortId then this process can have storage
        // permissions. Otherwise we get null permissions.
        match process.short_app_id() {
            ShortId::Fixed(id) => {
                // Check if we can get permissions from the TBF. If so, use
                // those, otherwise default to "individual" (ie can only write
                // its own state) permissions.
                if process.get_tbf_storage_permissions().is_some() {
                    self.tbf_permissions.get_permissions(process)
                } else {
                    StoragePermissions::new_self_only(id, &self.cap)
                }
            }
            ShortId::LocallyUnique => StoragePermissions::new_null(),
        }
    }
}
