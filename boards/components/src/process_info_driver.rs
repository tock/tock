// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Component for the process info capsule.

use capsules_extra::process_info_driver::{self, ProcessInfo};
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::capabilities::{ProcessManagementCapability, ProcessStartCapability};
use kernel::component::Component;
use kernel::create_capability;

#[macro_export]
macro_rules! process_info_component_static {
    ($C:ty $(,)?) => {{
        let process_info = kernel::static_buf!(
            capsules_extra::process_info_driver::ProcessInfo<
                $C,
            >
        );

        process_info
    };};
}

pub struct ProcessInfoComponent<C: ProcessManagementCapability + ProcessStartCapability> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    capability: C,
}

impl<C: ProcessManagementCapability + ProcessStartCapability> ProcessInfoComponent<C> {
    pub fn new(board_kernel: &'static kernel::Kernel, driver_num: usize, capability: C) -> Self {
        Self {
            board_kernel,
            driver_num,
            capability,
        }
    }
}

impl<C: ProcessManagementCapability + ProcessStartCapability + 'static> Component
    for ProcessInfoComponent<C>
{
    type StaticInput = &'static mut MaybeUninit<ProcessInfo<C>>;
    type Output = &'static process_info_driver::ProcessInfo<C>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let process_info = static_buffer.write(ProcessInfo::new(
            self.board_kernel,
            self.board_kernel.create_grant(self.driver_num, &grant_cap),
            self.capability,
        ));

        process_info
    }
}
