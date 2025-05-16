// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Shared implementations for ARM Cortex-M0+ MCUs.

#![no_std]

use core::fmt::Write;

pub mod mpu {
    use kernel::utilities::StaticRef;

    pub type MPU = cortexm::mpu::MPU<8, 256>;

    const MPU_BASE_ADDRESS: StaticRef<cortexm::mpu::MpuRegisters> =
        unsafe { StaticRef::new(0xE000ED90 as *const cortexm::mpu::MpuRegisters) };

    pub unsafe fn new() -> MPU {
        MPU::new(MPU_BASE_ADDRESS)
    }
}

// Re-export the base generic cortex-m functions here as they are
// valid on cortex-m0.
pub use cortexm::support;

pub use cortexm::initialize_ram_jump_to_main;
pub use cortexm::interrupt_mask;
pub use cortexm::nvic;
pub use cortexm::scb;
pub use cortexm::systick;
pub use cortexm::unhandled_interrupt;
pub use cortexm::CortexMVariant;
use cortexm0::CortexM0;

// Mock implementation for tests on Travis-CI.
#[cfg(not(any(doc, all(target_arch = "arm", target_os = "none"))))]
pub unsafe extern "C" fn svc_handler_m0p() {
    unimplemented!()
}

#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
extern "C" {
    pub fn svc_handler_m0p();
}

#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
core::arch::global_asm!(
    "
  .section .svc_handler_m0p, \"ax\"
  .global svc_handler_m0p
  .thumb_func
svc_handler_m0p:
  ldr r0, 100f // EXC_RETURN_MSP
  cmp lr, r0
  bne 300f // to_kernel

  // If we get here, then this is a context switch from the kernel to the
  // application. Set thread mode to unprivileged to run the application.
  movs r0, #1
  msr CONTROL, r0
  ldr r1, 200f // EXC_RETURN_PSP
  bx r1

300: // to_kernel
  ldr r0, =SYSCALL_FIRED
  movs r1, #1
  str r1, [r0, #0]
  // Set thread mode to privileged as we switch back to the kernel.
  movs r0, #0
  msr CONTROL, r0
  ldr r1, 100f // EXC_RETURN_MSP
  bx r1

.align 4
100: // EXC_RETURN_MSP
  .word 0xFFFFFFF9
200: // EXC_RETURN_PSP
  .word 0xFFFFFFFD
  "
);

// Enum with no variants to ensure that this type is not instantiable. It is
// only used to pass architecture-specific constants and functions via the
// `CortexMVariant` trait.
pub enum CortexM0P {}

impl cortexm::CortexMVariant for CortexM0P {
    const GENERIC_ISR: unsafe extern "C" fn() = CortexM0::GENERIC_ISR;
    const SYSTICK_HANDLER: unsafe extern "C" fn() = CortexM0::SYSTICK_HANDLER;
    const SVC_HANDLER: unsafe extern "C" fn() = svc_handler_m0p;
    const HARD_FAULT_HANDLER: unsafe extern "C" fn() = CortexM0::HARD_FAULT_HANDLER;

    #[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
    unsafe fn switch_to_user(
        user_stack: *const usize,
        process_regs: &mut [usize; 8],
    ) -> *const usize {
        CortexM0::switch_to_user(user_stack, process_regs)
    }

    #[cfg(not(any(doc, all(target_arch = "arm", target_os = "none"))))]
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
    pub type SysCall = cortexm::syscall::SysCall<crate::CortexM0P>;
}
