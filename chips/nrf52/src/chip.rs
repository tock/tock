// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::fmt::Write;
use cortexm4f::{nvic, CortexM4F, CortexMVariant};
use kernel::platform::chip::InterruptService;

pub struct NRF52<'a, I: InterruptService + 'a> {
    mpu: cortexm4f::mpu::MPU,
    userspace_kernel_boundary: cortexm4f::syscall::SysCall,
    interrupt_service: &'a I,
}

impl<'a, I: InterruptService + 'a> NRF52<'a, I> {
    pub unsafe fn new(interrupt_service: &'a I) -> Self {
        Self {
            mpu: cortexm4f::mpu::new(),
            userspace_kernel_boundary: cortexm4f::syscall::SysCall::new(),
            interrupt_service,
        }
    }
}

impl<'a, I: InterruptService + 'a> kernel::platform::chip::Chip for NRF52<'a, I> {
    type MPU = cortexm4f::mpu::MPU;
    type UserspaceKernelBoundary = cortexm4f::syscall::SysCall;
    type ThreadIdProvider = cortexm4f::thread_id::CortexMThreadIdProvider;

    fn mpu(&self) -> &Self::MPU {
        &self.mpu
    }

    fn userspace_kernel_boundary(&self) -> &Self::UserspaceKernelBoundary {
        &self.userspace_kernel_boundary
    }

    fn service_pending_interrupts(&self) {
        unsafe {
            while let Some(interrupt) = nvic::next_pending() {
                if !self.interrupt_service.service_interrupt(interrupt) {
                    panic!("unhandled interrupt {}", interrupt);
                }
                let n = nvic::Nvic::new(interrupt);
                n.clear_pending();
                n.enable();
            }
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        unsafe { nvic::has_pending() }
    }

    fn sleep(&self) {
        unsafe {
            cortexm4f::support::wfi();
        }
    }

    unsafe fn with_interrupts_disabled<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        cortexm4f::support::with_interrupts_disabled(f)
    }

    unsafe fn print_state(&self, write: &mut dyn Write) {
        CortexM4F::print_cortexm_state(write);
    }
}
