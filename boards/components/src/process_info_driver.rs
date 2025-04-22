// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Component for the process info capsule.

use capsules_extra::process_info_driver::{self, ProcessInfo};
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;

#[macro_export]
macro_rules! process_info_component_static {
    () => {{
        let process_info = kernel::static_buf!(
            capsules_extra::process_info_driver::ProcessInfo<
                components::process_info_driver::Capability,
            >
        );

        process_info
    };};
}

pub struct ProcessInfoComponent {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
}

impl ProcessInfoComponent {
    pub fn new(board_kernel: &'static kernel::Kernel, driver_num: usize) -> Self {
        Self {
            board_kernel,
            driver_num,
        }
    }
}

pub struct Capability;
unsafe impl capabilities::ProcessManagementCapability for Capability {}

impl Component for ProcessInfoComponent {
    type StaticInput = (&'static mut MaybeUninit<ProcessInfo<Capability>>,);
    type Output = &'static process_info_driver::ProcessInfo<Capability>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let process_info = static_buffer.0.write(ProcessInfo::new(
            self.board_kernel,
            self.board_kernel.create_grant(self.driver_num, &grant_cap),
            Capability,
        ));

        process_info
    }
}
