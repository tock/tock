// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Component for the IPC Relay Request capsule.
//!
//! Usage
//! -----
//! ```rust
//! let ipc_relay_request = IpcRelayRequestComponent::new(
//!     board_kernel,
//!     capsules_core::ipc::ipc_relay_request::DRIVER_NUM,
//!     create_capability!(capabilities::MemoryAllocationCapability)
//!     )
//!     .finalize(components::ipc_relay_request_component_static!());
//! ```

use capsules_core::ipc::ipc_relay_request::IpcRelayRequest;
use core::mem::MaybeUninit;
use kernel::capabilities::MemoryAllocationCapability;
use kernel::component::Component;

#[macro_export]
macro_rules! ipc_relay_request_component_static {
    () => {{ kernel::static_buf!(capsules_core::ipc::ipc_relay_request::IpcRelayRequest) }};
}

pub type IpcRelayRequestComponentType = capsules_core::ipc::ipc_relay_request::IpcRelayRequest;

pub struct IpcRelayRequestComponent<CAP: MemoryAllocationCapability> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    mem_cap: CAP,
}

impl<CAP: MemoryAllocationCapability> IpcRelayRequestComponent<CAP> {
    pub fn new(board_kernel: &'static kernel::Kernel, driver_num: usize, mem_cap: CAP) -> Self {
        Self {
            board_kernel,
            driver_num,
            mem_cap,
        }
    }
}

impl<CAP: MemoryAllocationCapability> Component for IpcRelayRequestComponent<CAP> {
    type StaticInput = &'static mut MaybeUninit<IpcRelayRequest>;
    type Output = &'static IpcRelayRequest;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        s.write(IpcRelayRequest::new(
            self.board_kernel
                .create_grant(self.driver_num, &self.mem_cap),
        ))
    }
}
