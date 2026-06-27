// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Component for the SyscallReturnTest capsule.
//!
//! Capsule: capsules/extra/src/syscall_return_test.rs

use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;

#[macro_export]
macro_rules! syscall_return_test_component_static {
    () => {{
        kernel::static_buf!(capsules_extra::syscall_return_test::SyscallReturnTest)
    };};
}

pub type SyscallReturnTestComponentType = capsules_extra::syscall_return_test::SyscallReturnTest;

pub struct SyscallReturnTestComponent {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
}

impl SyscallReturnTestComponent {
    pub fn new(board_kernel: &'static kernel::Kernel, driver_num: usize) -> Self {
        SyscallReturnTestComponent {
            board_kernel,
            driver_num,
        }
    }
}

impl Component for SyscallReturnTestComponent {
    type StaticInput =
        &'static mut MaybeUninit<capsules_extra::syscall_return_test::SyscallReturnTest>;
    type Output = &'static capsules_extra::syscall_return_test::SyscallReturnTest;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let grant = self.board_kernel.create_grant(self.driver_num, &grant_cap);
        s.write(capsules_extra::syscall_return_test::SyscallReturnTest::new(grant))
    }
}
