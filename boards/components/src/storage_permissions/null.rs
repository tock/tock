// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Component for creating a storage permissions policy that provides no storage
//! permissions.

use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::platform::chip::Chip;

#[macro_export]
macro_rules! storage_permissions_null_component_static {
    ($C:ty $(,)?) => {{
        kernel::static_buf!(capsules_system::storage_permissions::null::NullStoragePermissions<$C>)
    };};
}

pub type StoragePermissionsNullComponentType<C> =
    capsules_system::storage_permissions::null::NullStoragePermissions<C>;

pub struct StoragePermissionsNullComponent<C: Chip> {
    _chip: core::marker::PhantomData<C>,
}

impl<C: Chip> StoragePermissionsNullComponent<C> {
    pub fn new() -> Self {
        Self {
            _chip: core::marker::PhantomData,
        }
    }
}

impl<C: Chip + 'static> Component for StoragePermissionsNullComponent<C> {
    type StaticInput = &'static mut MaybeUninit<
        capsules_system::storage_permissions::null::NullStoragePermissions<C>,
    >;
    type Output = &'static capsules_system::storage_permissions::null::NullStoragePermissions<C>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        s.write(capsules_system::storage_permissions::null::NullStoragePermissions::new())
    }
}
