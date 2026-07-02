// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Component for the process info capsule.

use capsules_extra::process_info_driver::{self, ProcessInfo};
use core::mem::MaybeUninit;
use kernel::capabilities::MemoryAllocationCapability;
use kernel::capabilities::{ProcessManagementCapability, ProcessStartCapability};
use kernel::component::Component;

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

pub struct ProcessInfoComponent<
    C: ProcessManagementCapability + ProcessStartCapability,
    CAP: MemoryAllocationCapability + 'static,
> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    capability: C,
    mem_cap: CAP,
}

impl<
        C: ProcessManagementCapability + ProcessStartCapability,
        CAP: MemoryAllocationCapability + 'static,
    > ProcessInfoComponent<C, CAP>
{
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        capability: C,
        mem_cap: CAP,
    ) -> Self {
        Self {
            board_kernel,
            driver_num,
            capability,
            mem_cap,
        }
    }
}

impl<
        C: ProcessManagementCapability + ProcessStartCapability + 'static,
        CAP: MemoryAllocationCapability + 'static,
    > Component for ProcessInfoComponent<C, CAP>
{
    type StaticInput = &'static mut MaybeUninit<ProcessInfo<C>>;
    type Output = &'static process_info_driver::ProcessInfo<C>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let process_info = static_buffer.write(ProcessInfo::new(
            self.board_kernel,
            self.board_kernel
                .create_grant(self.driver_num, &self.mem_cap),
            self.capability,
        ));

        process_info
    }
}
