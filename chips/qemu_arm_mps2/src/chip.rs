// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Top-level chip definition for the ARM MPS2 ANXXX FPGA images.

use core::fmt::Write;

use cortexm::CortexMVariant;
use kernel::platform::chip::InterruptService;
use kernel::utilities::StaticRef;

const MPU_BASE_ADDRESS: StaticRef<cortexm::mpu::MpuRegisters> =
    unsafe { StaticRef::new(0xE000_ED90 as *const cortexm::mpu::MpuRegisters) };

pub type Mps2Mpu = cortexm::mpu::MPU<8, 32>;

pub struct QemuArmMps2Chip<'a, V: CortexMVariant, I: InterruptService + 'a> {
    mpu: Mps2Mpu,
    userspace_kernel_boundary: cortexm::syscall::SysCall<V>,
    interrupt_service: &'a I,
}

impl<'a, V: CortexMVariant, I: InterruptService + 'a> QemuArmMps2Chip<'a, V, I> {
    /// # Safety
    ///
    /// Must only be called once, as it takes ownership of the MPU and
    /// syscall-boundary hardware state.
    pub unsafe fn new(interrupt_service: &'a I) -> Self {
        Self {
            mpu: unsafe { Mps2Mpu::new(MPU_BASE_ADDRESS) },
            userspace_kernel_boundary: unsafe { cortexm::syscall::SysCall::new() },
            interrupt_service,
        }
    }
}

impl<'a, V: CortexMVariant, I: InterruptService + 'a> kernel::platform::chip::Chip
    for QemuArmMps2Chip<'a, V, I>
{
    type MPU = Mps2Mpu;
    type UserspaceKernelBoundary = cortexm::syscall::SysCall<V>;
    type ThreadIdProvider = cortexm::thread_id::CortexMThreadIdProvider;

    fn init() {
        // This board has no bootloader relocating the vector table, and no
        // documented silicon errata to work around (this is a QEMU-only
        // FPGA reference image, not real silicon), so there is nothing to
        // do beyond unmasking interrupts at the NVIC.
        cortexm::nvic::enable_all();
    }

    fn mpu(&self) -> &Self::MPU {
        &self.mpu
    }

    fn userspace_kernel_boundary(&self) -> &Self::UserspaceKernelBoundary {
        &self.userspace_kernel_boundary
    }

    fn service_pending_interrupts(&self) {
        while let Some(interrupt) = cortexm::nvic::next_pending() {
            if !self.interrupt_service.service_interrupt(interrupt) {
                panic!("unhandled interrupt {}", interrupt);
            }
            let n = cortexm::nvic::Nvic::new(interrupt);
            n.clear_pending();
            n.enable();
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        cortexm::nvic::has_pending()
    }

    fn sleep(&self) {
        unsafe {
            cortexm::support::wfi();
        }
    }

    fn with_interrupts_disabled<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        cortexm::support::with_interrupts_disabled(f)
    }

    unsafe fn print_state(_this: Option<&Self>, write: &mut dyn Write) {
        unsafe {
            V::print_cortexm_state(write);
        }
    }
}
