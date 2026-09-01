// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Component for the SyscallReturnTest capsule.
//!
//! Capsule: capsules/extra/src/syscall_return_test.rs

use core::mem::MaybeUninit;
use kernel::capabilities::MemoryAllocationCapability;
use kernel::component::Component;

#[macro_export]
macro_rules! syscall_return_test_component_static {
    () => {{ kernel::static_buf!(capsules_extra::syscall_return_test::SyscallReturnTest) }};
}

pub type SyscallReturnTestComponentType = capsules_extra::syscall_return_test::SyscallReturnTest;

pub struct SyscallReturnTestComponent<CAP: MemoryAllocationCapability + 'static> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    mem_cap: CAP,
}

impl<CAP: MemoryAllocationCapability + 'static> SyscallReturnTestComponent<CAP> {
    pub fn new(board_kernel: &'static kernel::Kernel, driver_num: usize, mem_cap: CAP) -> Self {
        Self {
            board_kernel,
            driver_num,
            mem_cap,
        }
    }
}

impl<CAP: MemoryAllocationCapability + 'static> Component for SyscallReturnTestComponent<CAP> {
    type StaticInput =
        &'static mut MaybeUninit<capsules_extra::syscall_return_test::SyscallReturnTest>;
    type Output = &'static capsules_extra::syscall_return_test::SyscallReturnTest;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let grant = self
            .board_kernel
            .create_grant(self.driver_num, &self.mem_cap);
        s.write(capsules_extra::syscall_return_test::SyscallReturnTest::new(
            grant,
        ))
    }
}
