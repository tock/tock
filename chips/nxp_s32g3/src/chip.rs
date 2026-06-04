// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use core::fmt::Write;
use cortexm7::{CortexM7, CortexMVariant};
use kernel::platform::chip::{Chip, InterruptService};

pub struct S32g3<I: InterruptService + 'static> {
    mpu: cortexm7::mpu::MPU,
    userspace_kernel_boundary: cortexm7::syscall::SysCall,
    interrupt_service: &'static I,
}

impl<I: InterruptService + 'static> S32g3<I> {
    pub unsafe fn new(interrupt_service: &'static I) -> Self {
        Self {
            mpu: cortexm7::mpu::new(),
            userspace_kernel_boundary: cortexm7::syscall::SysCall::new(),
            interrupt_service,
        }
    }
}

impl<I: InterruptService + 'static> Chip for S32g3<I> {
    type MPU = cortexm7::mpu::MPU;
    type UserspaceKernelBoundary = cortexm7::syscall::SysCall;
    type ThreadIdProvider = cortexm7::thread_id::CortexMThreadIdProvider;

    fn init() {
        // This is called exactly once from the board's `start()`,
        // before interrupts are enabled and before any kernel structures
        // are active. Each `unsafe` block below justifies its own soundness.

        // # Safety
        //
        // Reset NVIC state before installing our vector table. This is sound
        // because we are in the board's `start()`, called once before any
        // interrupts are enabled.
        unsafe {
            cortexm7::nvic::disable_all();
            cortexm7::nvic::clear_all_pending();
        }

        let vector_table: *const [unsafe extern "C" fn(); 16] =
            core::ptr::addr_of!(crate::BASE_VECTORS);
        let vector_table: *const () = vector_table.cast();

        // # Safety
        //
        // BASE_VECTORS is the correct vector table for this S32G3 image.
        // This is sound because we are on S32G3 hardware running this image.
        unsafe {
            cortexm7::scb::set_vector_table_offset(vector_table);
        }

        // # Safety
        //
        // Re-enable NVIC so peripheral drivers can arm their interrupts.
        // This is sound because the vector table is now correctly installed.
        unsafe {
            cortexm7::nvic::enable_all();
        }
    }

    fn service_pending_interrupts(&self) {
        unsafe {
            while let Some(interrupt) = cortexm7::nvic::next_pending() {
                let handled = self.interrupt_service.service_interrupt(interrupt);
                let nvic = cortexm7::nvic::Nvic::new(interrupt);
                if !handled {
                    panic!("Unhandled interrupt {}", interrupt);
                } else {
                    nvic.clear_pending();
                    nvic.enable();
                }
            }
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        unsafe { cortexm7::nvic::has_pending() }
    }

    fn mpu(&self) -> &Self::MPU {
        &self.mpu
    }

    fn userspace_kernel_boundary(&self) -> &Self::UserspaceKernelBoundary {
        &self.userspace_kernel_boundary
    }

    fn sleep(&self) {
        unsafe {
            cortexm7::scb::unset_sleepdeep();
            cortexm7::support::wfi();
        }
    }

    unsafe fn with_interrupts_disabled<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        cortexm7::support::with_interrupts_disabled(f)
    }

    unsafe fn print_state(_chip: Option<&Self>, writer: &mut dyn Write) {
        CortexM7::print_cortexm_state(writer);
    }
}
