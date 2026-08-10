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
//!     PMCapability,
//!     create_capability!(capabilities::MemoryAllocationCapability)
//!     )
//!     .finalize(components::ipc_registry_package_name_component_static!());
//! ```

use capsules_core::ipc::ipc_registry_package_name::IpcRegistryPackageName;
use core::mem::MaybeUninit;
use kernel::capabilities::{MemoryAllocationCapability, ProcessManagementCapability};
use kernel::component::Component;

#[macro_export]
macro_rules! ipc_registry_package_name_component_static {
    ($C:ty $(,)?) => {{
        kernel::static_buf!(
            capsules_core::ipc::ipc_registry_package_name::IpcRegistryPackageName<
                $C,
            >
        )
    }};
}

pub type IpcRegistryPackageNameComponentType<C> =
    capsules_core::ipc::ipc_registry_package_name::IpcRegistryPackageName<C>;

pub struct IpcRegistryPackageNameComponent<
    C: ProcessManagementCapability,
    CAP: MemoryAllocationCapability,
> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    capability: C,
    mem_cap: CAP,
    validation: Option<capsules_core::ipc::ipc_registry_package_name::ValidationFunction<C>>,
}

impl<C: ProcessManagementCapability, CAP: MemoryAllocationCapability>
    IpcRegistryPackageNameComponent<C, CAP>
{
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        capability: C,
        mem_cap: CAP,
        validation: Option<capsules_core::ipc::ipc_registry_package_name::ValidationFunction<C>>,
    ) -> Self {
        Self {
            board_kernel,
            driver_num,
            capability,
            mem_cap,
            validation,
        }
    }
}

impl<C: ProcessManagementCapability + 'static, CAP: MemoryAllocationCapability> Component
    for IpcRegistryPackageNameComponent<C, CAP>
{
    type StaticInput = &'static mut MaybeUninit<IpcRegistryPackageName<C>>;
    type Output = &'static IpcRegistryPackageName<C>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        s.write(IpcRegistryPackageName::new(
            self.board_kernel
                .create_grant(self.driver_num, &self.mem_cap),
            self.board_kernel,
            self.capability,
            self.validation,
        ))
    }
}
