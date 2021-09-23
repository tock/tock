//! Shared implementations for ARM Cortex-M0+ MCUs.

#![crate_name = "cortexm0p"]
#![crate_type = "rlib"]
#![feature(asm)]
#![feature(naked_functions)]
#![no_std]

pub mod mpu {
    pub type MPU = cortexm::mpu::MPU<8, 256>;
}

// Re-export the base generic cortex-m functions here as they are
// valid on cortex-m0.
pub use cortexm::support;

pub use cortexm::initialize_ram_jump_to_main;
pub use cortexm::interrupt_mask;
pub use cortexm::nvic;
pub use cortexm::print_cortexm_state as print_cortexm0_state;
pub use cortexm::scb;
pub use cortexm::syscall;
pub use cortexm::systick;
pub use cortexm::unhandled_interrupt;
pub use cortexm0::generic_isr;
pub use cortexm0::hard_fault_handler;
pub use cortexm0::systick_handler;

// Mock implementation for tests on Travis-CI.
#[cfg(not(any(target_arch = "arm", target_os = "none")))]
pub unsafe extern "C" fn switch_to_user(
    _user_stack: *const u8,
    _process_regs: &mut [usize; 8],
) -> *mut u8 {
    unimplemented!()
}

// Mock implementation for tests on Travis-CI.
#[cfg(not(any(target_arch = "arm", target_os = "none")))]
pub unsafe extern "C" fn svc_handler() {
    unimplemented!()
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
#[naked]
pub unsafe extern "C" fn svc_handler() {
    asm!(
        "
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
  ",
        options(noreturn)
    );
}
