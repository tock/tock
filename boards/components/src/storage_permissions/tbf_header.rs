// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Component for creating a storage permissions policy that grants applications
//! storage permissions based on TBF headers.

use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::platform::chip::Chip;

#[macro_export]
macro_rules! storage_permissions_tbf_header_component_static {
    ($C:ty $(,)?) => {{
        kernel::static_buf!(
            capsules_system::storage_permissions::tbf_header::TbfHeaderStoragePermissions<$C>
        )
    };};
}

pub type StoragePermissionsTbfHeaderComponentType<C> =
    capsules_system::storage_permissions::tbf_header::TbfHeaderStoragePermissions<C>;

pub struct StoragePermissionsTbfHeaderComponent<C: Chip> {
    _chip: core::marker::PhantomData<C>,
}

impl<C: Chip> StoragePermissionsTbfHeaderComponent<C> {
    pub fn new() -> Self {
        Self {
            _chip: core::marker::PhantomData,
        }
    }
}

impl<C: Chip + 'static> Component for StoragePermissionsTbfHeaderComponent<C> {
    type StaticInput = &'static mut MaybeUninit<
        capsules_system::storage_permissions::tbf_header::TbfHeaderStoragePermissions<C>,
    >;
    type Output =
        &'static capsules_system::storage_permissions::tbf_header::TbfHeaderStoragePermissions<C>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        s.write(
            capsules_system::storage_permissions::tbf_header::TbfHeaderStoragePermissions::new(),
        )
    }
}
