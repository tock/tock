// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Component for the IPC Registry Package Name capsule.
//!
//! Usage
//! -----
//! ```rust
//! pub struct PMCapability;
//! unsafe impl capabilities::ProcessManagementCapability for PMCapability {}
//!
//! let ipc_registry_package_name = IpcRegistryPackageNameComponent::new(
//!     board_kernel,
//!     capsules_core::ipc::ipc_registry_package_name::DRIVER_NUM,
//!     &capsules_core::ipc::filters::IpcPackageNameRegistrationFilterNull {},
//!     PMCapability,
//!     create_capability!(capabilities::MemoryAllocationCapability)
//!     )
//!     .finalize(components::ipc_registry_package_name_component_static!());
//! ```

use capsules_core::ipc::ipc_registry_package_name::IpcRegistryPackageName;
use core::mem::MaybeUninit;
use kernel::capabilities::{MemoryAllocationCapability, ProcessManagementCapability};
use kernel::component::Component;
use kernel::platform::registration::RegistrationFilter;

#[macro_export]
macro_rules! ipc_registry_package_name_component_static {
    ($RF:ty, $C:ty $(,)?) => {{
        kernel::static_buf!(
            capsules_core::ipc::ipc_registry_package_name::IpcRegistryPackageName<'static, $RF, $C>
        )
    }};
}

pub type IpcRegistryPackageNameComponentType<RF, C> =
    capsules_core::ipc::ipc_registry_package_name::IpcRegistryPackageName<'static, RF, C>;

pub struct IpcRegistryPackageNameComponent<
    RF: RegistrationFilter + 'static,
    C: ProcessManagementCapability,
    CAP: MemoryAllocationCapability,
> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    registration_filter: &'static RF,
    capability: C,
    mem_cap: CAP,
}

impl<RF: RegistrationFilter, C: ProcessManagementCapability, CAP: MemoryAllocationCapability>
    IpcRegistryPackageNameComponent<RF, C, CAP>
{
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        registration_filter: &'static RF,
        capability: C,
        mem_cap: CAP,
    ) -> Self {
        Self {
            board_kernel,
            driver_num,
            registration_filter,
            capability,
            mem_cap,
        }
    }
}

impl<
    RF: RegistrationFilter<RegistrationIdentifier = &'static str>,
    C: ProcessManagementCapability + 'static,
    CAP: MemoryAllocationCapability,
> Component for IpcRegistryPackageNameComponent<RF, C, CAP>
{
    type StaticInput = &'static mut MaybeUninit<IpcRegistryPackageName<'static, RF, C>>;
    type Output = &'static IpcRegistryPackageName<'static, RF, C>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        s.write(IpcRegistryPackageName::new(
            self.board_kernel
                .create_grant(self.driver_num, &self.mem_cap),
            self.registration_filter,
            self.board_kernel,
            self.capability,
        ))
    }
}
