// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Component for creating a storage permissions policy that grants applications
//! access to their own stored state.
//!
//! ```rust
//! let storage_permissions_policy =
//!     components::storage_permissions::individual::StoragePermissionsIndividualComponent::new()
//!         .finalize(
//!             components::storage_permissions_individual_component_static!(nrf52840dk_lib::Chip),
//!         );
//! ```

use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::platform::chip::Chip;
use kernel::process::ProcessStandardDebug;

#[macro_export]
macro_rules! storage_permissions_individual_component_static {
    ($C:ty, $D:ty $(,)?) => {{
        kernel::static_buf!(
            capsules_system::storage_permissions::individual::IndividualStoragePermissions<
                $C,
                $D,
                components::storage_permissions::individual::AppStoreCapability
            >
        )
    };};
}

pub struct AppStoreCapability;
unsafe impl capabilities::ApplicationStorageCapability for AppStoreCapability {}

pub type StoragePermissionsIndividualComponentType<C, D> =
    capsules_system::storage_permissions::individual::IndividualStoragePermissions<
        C,
        D,
        AppStoreCapability,
    >;

pub struct StoragePermissionsIndividualComponent<C: Chip, D: ProcessStandardDebug> {
    _chip: core::marker::PhantomData<C>,
    _debug: core::marker::PhantomData<D>,
}

impl<C: Chip, D: ProcessStandardDebug> StoragePermissionsIndividualComponent<C, D> {
    pub fn new() -> Self {
        Self {
            _chip: core::marker::PhantomData,
            _debug: core::marker::PhantomData,
        }
    }
}

impl<C: Chip + 'static, D: ProcessStandardDebug + 'static> Component
    for StoragePermissionsIndividualComponent<C, D>
{
    type StaticInput = &'static mut MaybeUninit<
        capsules_system::storage_permissions::individual::IndividualStoragePermissions<
            C,
            D,
            AppStoreCapability,
        >,
    >;
    type Output =
        &'static capsules_system::storage_permissions::individual::IndividualStoragePermissions<
            C,
            D,
            AppStoreCapability,
        >;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        s.write(
            capsules_system::storage_permissions::individual::IndividualStoragePermissions::new(
                AppStoreCapability,
            ),
        )
    }
}
