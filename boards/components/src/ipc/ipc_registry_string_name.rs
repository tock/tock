// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Component for the IPC Registry String Name capsule.
//!
//! Usage
//! -----
//! ```rust
//! let ipc_registry_string_name = IpcRegistryStringNameComponent::new(
//!     board_kernel,
//!     capsules_core::ipc::ipc_registry_string_name::DRIVER_NUM,
//!     create_capability!(capabilities::MemoryAllocationCapability)
//!     )
//!     .finalize(components::ipc_registry_string_name_component_static!());
//! ```

use capsules_core::ipc::ipc_registry_string_name::IpcRegistryStringName;
use core::mem::MaybeUninit;
use kernel::capabilities::MemoryAllocationCapability;
use kernel::component::Component;

#[macro_export]
macro_rules! ipc_registry_string_name_component_static {
    () => {{ kernel::static_buf!(capsules_core::ipc::ipc_registry_string_name::IpcRegistryStringName) }};
}

pub type IpcRegistryStringNameComponentType =
    capsules_core::ipc::ipc_registry_string_name::IpcRegistryStringName;

pub struct IpcRegistryStringNameComponent<CAP: MemoryAllocationCapability> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    mem_cap: CAP,
    validation:
        Option<capsules_core::ipc::ipc_registration_validation::IpcRegistrationValidationFunction>,
}

impl<CAP: MemoryAllocationCapability> IpcRegistryStringNameComponent<CAP> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        mem_cap: CAP,
        validation: Option<
            capsules_core::ipc::ipc_registration_validation::IpcRegistrationValidationFunction,
        >,
    ) -> Self {
        Self {
            board_kernel,
            driver_num,
            mem_cap,
            validation,
        }
    }
}

impl<CAP: MemoryAllocationCapability> Component for IpcRegistryStringNameComponent<CAP> {
    type StaticInput = &'static mut MaybeUninit<IpcRegistryStringName>;
    type Output = &'static IpcRegistryStringName;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        s.write(IpcRegistryStringName::new(
            self.board_kernel
                .create_grant(self.driver_num, &self.mem_cap),
            self.validation,
        ))
    }
}
