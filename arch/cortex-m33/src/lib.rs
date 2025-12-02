// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Shared implementations for ARM Cortex-M33 MCUs.

#![no_std]

use core::fmt::Write;

pub mod mpu_v8m;

pub mod mpu {
    use crate::mpu_v8m;
    use kernel::utilities::StaticRef;

    // Note: Cortex-M33 supports up to 16 regions.
    pub type MPU<const NUM_REGIONS: usize> = mpu_v8m::MPU<NUM_REGIONS>;

    const MPU_BASE_ADDRESS: StaticRef<crate::mpu_v8m::MpuRegisters> =
        unsafe { StaticRef::new(0xE000ED90 as *const crate::mpu_v8m::MpuRegisters) };

    pub unsafe fn new<const NUM_REGIONS: usize>() -> mpu_v8m::MPU<NUM_REGIONS> {
        mpu_v8m::MPU::new(MPU_BASE_ADDRESS)
    }
}

pub use cortexm::initialize_ram_jump_to_main;
pub use cortexm::interrupt_mask;
pub use cortexm::nvic;
pub use cortexm::scb;
pub use cortexm::support;
pub use cortexm::systick;
pub use cortexm::thread_id;
pub use cortexm::unhandled_interrupt;
pub use cortexm::CortexMVariant;

// Enum with no variants to ensure that this type is not instantiable. It is
// only used to pass architecture-specific constants and functions via the
// `CortexMVariant` trait.
pub enum CortexM33 {}

impl cortexm::CortexMVariant for CortexM33 {
    const GENERIC_ISR: unsafe extern "C" fn() = cortexv7m::generic_isr_arm_v7m;
    const SYSTICK_HANDLER: unsafe extern "C" fn() = cortexv7m::systick_handler_arm_v7m;
    const SVC_HANDLER: unsafe extern "C" fn() = cortexv7m::svc_handler_arm_v7m;
    const HARD_FAULT_HANDLER: unsafe extern "C" fn() = cortexv7m::hard_fault_handler_arm_v7m;

    #[cfg(all(target_arch = "arm", target_os = "none"))]
    unsafe fn switch_to_user(
        user_stack: *const usize,
        process_regs: &mut [usize; 8],
    ) -> *const usize {
        cortexv7m::switch_to_user_arm_v7m(user_stack, process_regs)
    }

    #[cfg(not(all(target_arch = "arm", target_os = "none")))]
    unsafe fn switch_to_user(
        _user_stack: *const usize,
        _process_regs: &mut [usize; 8],
    ) -> *const usize {
        unimplemented!()
    }

    #[inline]
    unsafe fn print_cortexm_state(writer: &mut dyn Write) {
        cortexm::print_cortexm_state(writer)
    }
}

pub mod syscall {
    pub type SysCall = cortexm::syscall::SysCall<crate::CortexM33>;
}
