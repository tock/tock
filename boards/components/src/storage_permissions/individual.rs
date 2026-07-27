// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Component for creating a storage permissions policy that grants applications
//! access to their own stored state.
//!
//! ```rust
//! kernel::declare_capability!(AppStoreCap: kernel::capabilities::ApplicationStorageCapability);
//! let storage_permissions_policy =
//!     components::storage_permissions::individual::StoragePermissionsIndividualComponent::new(
//!         AppStoreCap,
//!     )
//!     .finalize(
//!         components::storage_permissions_individual_component_static!(
//!             nrf52840dk_lib::Chip,
//!             kernel::process::ProcessStandardDebugFull,
//!             AppStoreCap,
//!         ),
//!     );
//! ```

use core::mem::MaybeUninit;
use kernel::capabilities::ApplicationStorageCapability;
use kernel::component::Component;
use kernel::platform::chip::Chip;
use kernel::process::ProcessStandardDebug;

#[macro_export]
macro_rules! storage_permissions_individual_component_static {
    ($C:ty, $D:ty, $CAP:ty $(,)?) => {{
        kernel::static_buf!(
            capsules_system::storage_permissions::individual::IndividualStoragePermissions<
                $C,
                $D,
                $CAP,
            >
        )
    };};
}

pub type StoragePermissionsIndividualComponentType<C, D, CAP> =
    capsules_system::storage_permissions::individual::IndividualStoragePermissions<C, D, CAP>;

pub struct StoragePermissionsIndividualComponent<
    C: Chip,
    D: ProcessStandardDebug,
    CAP: ApplicationStorageCapability + 'static,
> {
    cap: CAP,
    _chip: core::marker::PhantomData<C>,
    _debug: core::marker::PhantomData<D>,
}

impl<C: Chip, D: ProcessStandardDebug, CAP: ApplicationStorageCapability>
    StoragePermissionsIndividualComponent<C, D, CAP>
{
    pub fn new(cap: CAP) -> Self {
        Self {
            cap,
            _chip: core::marker::PhantomData,
            _debug: core::marker::PhantomData,
        }
    }
}

impl<
    C: Chip + 'static,
    D: ProcessStandardDebug + 'static,
    CAP: ApplicationStorageCapability + 'static,
> Component for StoragePermissionsIndividualComponent<C, D, CAP>
{
    type StaticInput = &'static mut MaybeUninit<
        capsules_system::storage_permissions::individual::IndividualStoragePermissions<C, D, CAP>,
    >;
    type Output =
        &'static capsules_system::storage_permissions::individual::IndividualStoragePermissions<
            C,
            D,
            CAP,
        >;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        s.write(
            capsules_system::storage_permissions::individual::IndividualStoragePermissions::new(
                self.cap,
            ),
        )
    }
}
