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
//!     &capsules_core::ipc::filters::IpcStringNameRegistrationFilterNull {},
//!     create_capability!(capabilities::MemoryAllocationCapability)
//!     )
//!     .finalize(components::ipc_registry_string_name_component_static!());
//! ```

use capsules_core::ipc::ipc_registry_string_name::IpcRegistryStringName;
use core::mem::MaybeUninit;
use kernel::capabilities::MemoryAllocationCapability;
use kernel::component::Component;
use kernel::platform::registration::RegistrationFilter;

#[macro_export]
macro_rules! ipc_registry_string_name_component_static {
    ($RF:ty $(,)?) => {{
        kernel::static_buf!(
            capsules_core::ipc::ipc_registry_string_name::IpcRegistryStringName<'static, $RF>
        )
    }};
}

pub type IpcRegistryStringNameComponentType<RF> =
    capsules_core::ipc::ipc_registry_string_name::IpcRegistryStringName<'static, RF>;

pub struct IpcRegistryStringNameComponent<
    RF: RegistrationFilter + 'static,
    CAP: MemoryAllocationCapability,
> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    registration_filter: &'static RF,
    mem_cap: CAP,
}

impl<RF: RegistrationFilter, CAP: MemoryAllocationCapability>
    IpcRegistryStringNameComponent<RF, CAP>
{
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        registration_filter: &'static RF,
        mem_cap: CAP,
    ) -> Self {
        Self {
            board_kernel,
            driver_num,
            registration_filter,
            mem_cap,
        }
    }
}

impl<
    RF: RegistrationFilter<
        RegistrationIdentifier = [u8; capsules_core::ipc::ipc_registry_string_name::MAX_STRING_LEN],
    >,
    CAP: MemoryAllocationCapability,
> Component for IpcRegistryStringNameComponent<RF, CAP>
{
    type StaticInput = &'static mut MaybeUninit<IpcRegistryStringName<'static, RF>>;
    type Output = &'static IpcRegistryStringName<'static, RF>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        s.write(IpcRegistryStringName::new(
            self.board_kernel
                .create_grant(self.driver_num, &self.mem_cap),
            self.registration_filter,
        ))
    }
}
