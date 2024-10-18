// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

use core::cmp;
use kernel::capabilities::ApplicationStorageCapability;
use kernel::platform::chip::Chip;
use kernel::process::Process;
use kernel::process::ShortId;
use kernel::storage_permissions::StoragePermissions;

/// Assign storage permissions based on the fields in the application's TBF
/// header.
///
/// If the process does not have a fixed ShortId then it cannot have storage
/// permissions and will get null permissions.
///
/// If the header is _not_ present, then the process will be assigned null
/// permissions.
pub struct TbfHeaderStoragePermissions<
    C: Chip,
    D: kernel::process::ProcessStandardDebug,
    CAP: ApplicationStorageCapability,
> {
    cap: CAP,
    _chip: core::marker::PhantomData<C>,
    _debug: core::marker::PhantomData<D>,
}

impl<C: Chip, D: kernel::process::ProcessStandardDebug, CAP: ApplicationStorageCapability>
    TbfHeaderStoragePermissions<C, D, CAP>
{
    pub fn new(cap: CAP) -> Self {
        Self {
            cap,
            _chip: core::marker::PhantomData,
            _debug: core::marker::PhantomData,
        }
    }
}

impl<C: Chip, D: kernel::process::ProcessStandardDebug, CAP: ApplicationStorageCapability>
    kernel::process::ProcessStandardStoragePermissionsPolicy<C, D>
    for TbfHeaderStoragePermissions<C, D, CAP>
{
    fn get_permissions(
        &self,
        process: &kernel::process::ProcessStandard<C, D>,
    ) -> StoragePermissions {
        // If we have a fixed ShortId then this process can have storage
        // permissions. Otherwise we get null permissions.
        match process.short_app_id() {
            ShortId::Fixed(id) => {
                if let Some((write_allowed, read_count, read_ids, modify_count, modify_ids)) =
                    process.get_tbf_storage_permissions()
                {
                    let read_count_capped = cmp::min(read_count, 8);
                    let modify_count_capped = cmp::min(modify_count, 8);

                    StoragePermissions::new_fixed_size(
                        id,
                        write_allowed,
                        false,
                        read_count_capped,
                        read_ids,
                        modify_count_capped,
                        modify_ids,
                        &self.cap,
                    )
                } else {
                    StoragePermissions::new_null()
                }
            }
            ShortId::LocallyUnique => StoragePermissions::new_null(),
        }
    }
}
