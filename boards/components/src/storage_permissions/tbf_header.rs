// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Component for creating a storage permissions policy that grants applications
//! storage permissions based on TBF headers.

use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::platform::chip::Chip;
use kernel::process::ProcessStandardDebug;

#[macro_export]
macro_rules! storage_permissions_tbf_header_component_static {
    ($C:ty, $D:ty $(,)?) => {{
        kernel::static_buf!(
            capsules_system::storage_permissions::tbf_header::TbfHeaderStoragePermissions<
                $C,
                $D,
                components::storage_permissions::tbf_header::AppStoreCapability
            >
        )
    };};
}

pub struct AppStoreCapability;
unsafe impl kernel::capabilities::ApplicationStorageCapability for AppStoreCapability {}

pub type StoragePermissionsTbfHeaderComponentType<C, D> =
    capsules_system::storage_permissions::tbf_header::TbfHeaderStoragePermissions<
        C,
        D,
        AppStoreCapability,
    >;

pub struct StoragePermissionsTbfHeaderComponent<C: Chip, D: ProcessStandardDebug> {
    _chip: core::marker::PhantomData<C>,
    _debug: core::marker::PhantomData<D>,
}

impl<C: Chip, D: ProcessStandardDebug> StoragePermissionsTbfHeaderComponent<C, D> {
    pub fn new() -> Self {
        Self {
            _chip: core::marker::PhantomData,
            _debug: core::marker::PhantomData,
        }
    }
}

impl<C: Chip + 'static, D: ProcessStandardDebug + 'static> Component
    for StoragePermissionsTbfHeaderComponent<C, D>
{
    type StaticInput = &'static mut MaybeUninit<
        capsules_system::storage_permissions::tbf_header::TbfHeaderStoragePermissions<
            C,
            D,
            AppStoreCapability,
        >,
    >;
    type Output =
        &'static capsules_system::storage_permissions::tbf_header::TbfHeaderStoragePermissions<
            C,
            D,
            AppStoreCapability,
        >;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        s.write(
            capsules_system::storage_permissions::tbf_header::TbfHeaderStoragePermissions::new(
                AppStoreCapability,
            ),
        )
    }
}
