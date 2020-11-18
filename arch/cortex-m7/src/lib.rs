//! Shared implementations for ARM Cortex-M7 MCUs.

#![crate_name = "cortexm7"]
#![crate_type = "rlib"]
#![feature(llvm_asm, naked_functions)]
#![no_std]

pub mod mpu;

// Re-export the base generic cortex-m functions here as they are
// valid on cortex-m7.
pub use cortexm::support;

pub use cortexm::generic_isr;
pub use cortexm::hard_fault_handler_arm_v7m as hard_fault_handler;
pub use cortexm::nvic;
pub use cortexm::print_cortexm_state as print_cortexm7_state;
pub use cortexm::scb;
pub use cortexm::svc_handler;
pub use cortexm::syscall;
pub use cortexm::systick;
pub use cortexm::systick_handler;

/// Provide a `switch_to_user` function with exactly that name for syscall.rs.
#[cfg(all(target_arch = "arm", target_os = "none"))]
#[no_mangle]
pub unsafe extern "C" fn switch_to_user(
    user_stack: *const usize,
    process_regs: &mut [usize; 8],
) -> *const usize {
    cortexm::switch_to_user_arm_v7m(user_stack, process_regs)
}

#[cfg(not(any(target_arch = "arm", target_os = "none")))]
pub unsafe extern "C" fn switch_to_user(
    _user_stack: *const u8,
    _process_regs: &mut [usize; 8],
) -> *const usize {
    unimplemented!()
}
